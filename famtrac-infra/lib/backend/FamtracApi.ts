import { ArnFormat, Duration, Stack } from "aws-cdk-lib";
import { CfnApi, CfnApiMapping, CfnAuthorizer, CfnDomainName, CfnIntegration, CfnRoute, CfnRouteProps, CfnStage } from "aws-cdk-lib/aws-apigatewayv2";
import { ICertificate } from "aws-cdk-lib/aws-certificatemanager";
import { AttributeType, BillingMode, ITable, ProjectionType, Table } from "aws-cdk-lib/aws-dynamodb";
import { Effect, PolicyStatement, ServicePrincipal } from "aws-cdk-lib/aws-iam";
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
import { CnameRecord, IHostedZone } from "aws-cdk-lib/aws-route53";
import { Construct } from "constructs";


export interface FamtracApiDomainProps {
    readonly certificate: ICertificate;
    readonly hostedZone: IHostedZone;
    readonly domainName: string;
}

export interface FamtracApiAuthorizationProps {
    readonly issuer: string;
    readonly audience: string[];
    readonly scopes?: string[];
}

export interface IFamtracApi {
    readonly table: ITable;
    readonly apiId: string;
    readonly stageId: string;

    addDomain(id: string, props: FamtracApiDomainProps): void;
}

export interface FamtracApiProps {
    readonly apiName?: string;
    readonly table?: ITable;
    readonly enableDevelopmentOrigin?: boolean;
    readonly customOrigins?: string[];
    readonly authorization?: FamtracApiAuthorizationProps;
    readonly backendCode: Code;
}

export class FamtracApi extends Construct implements IFamtracApi {
    readonly table: ITable;
    readonly apiId: string;
    readonly stageId: string;

    constructor(scope: Construct, id: string, props: FamtracApiProps) {
        super(scope, id);

        let table = props.table;
        let indexes: string[] = [];
        if (!table) {
            let newTable = new Table(this, 'Data', {
                partitionKey: {
                    name: 'PK',
                    type: AttributeType.STRING,
                },
                sortKey: {
                    name: 'SK',
                    type: AttributeType.STRING,
                },
                readCapacity: 1,
                writeCapacity: 1,
                tableName: 'FamtracData',
                billingMode: BillingMode.PROVISIONED,
                timeToLiveAttribute: 'expires_in',
            });
            let indexName = "GSI-1";
            newTable.addGlobalSecondaryIndex({
                indexName,
                partitionKey: {
                    name: 'PK',
                    type: AttributeType.STRING,
                },
                sortKey: {
                    name: 'timestamp',
                    type: AttributeType.STRING,
                },
                projectionType: ProjectionType.ALL,
                readCapacity: 1,
                writeCapacity: 1,
            });
            indexes.push(indexName);
            table = newTable;
        }
        this.table = table;

        let backendFunction = new Function(this, 'BackendFunction', {
            code: props.backendCode,
            handler: 'bootstrap',
            runtime: Runtime.PROVIDED_AL2023,
            memorySize: 512,
            timeout: Duration.seconds(30),
            environment: {
                DYNAMODB_TABLE_NAME: this.table.tableName,
            },
            architecture: Architecture.X86_64,
        });

        backendFunction.addToRolePolicy(new PolicyStatement({
            effect: Effect.ALLOW,
            actions: [
                'dynamodb:GetItem',
                'dynamodb:PutItem',
                'dynamodb:UpdateItem',
                'dynamodb:DeleteItem',
                'dynamodb:Query',
            ],
            resources: [
                this.table.tableArn
            ]
        }));

        if (indexes.length > 0) {
            backendFunction.addToRolePolicy(new PolicyStatement({
                effect: Effect.ALLOW,
                actions: [
                    'dynamodb:Query',
                ],
                resources: indexes.map(indexName => `${this.table.tableArn}/index/${indexName}`),
            }));
        }

        let allowOrigins = [];
        if (props.enableDevelopmentOrigin === true) {
            allowOrigins.push('http://localhost:5173');
        }
        props.customOrigins?.forEach(origin => allowOrigins.push(origin));
        const apiName = props.apiName ?? 'famtrac-api';
        const api = new CfnApi(this, 'Api', {
            name: apiName,
            protocolType: 'HTTP',
            corsConfiguration: {
                allowCredentials: true,
                allowHeaders: [
                    'Content-Type',
                    'Content-Length',
                    'Accept',
                    'Authorization',
                ],
                allowMethods: [
                    'GET',
                    'POST',
                    'PUT',
                    'DELETE',
                    'OPTIONS',
                ],
                allowOrigins,
            },
            routeSelectionExpression: '$request.method $request.path',
        });
        this.apiId = api.ref;

        const resourceIntegration = new CfnIntegration(this, 'FamtracBackend', {
            apiId: this.apiId,
            integrationType: 'AWS_PROXY',
            connectionType: 'INTERNET',
            integrationMethod: 'POST',
            payloadFormatVersion: '2.0',
            timeoutInMillis: Duration.seconds(30).toMilliseconds(),
            integrationUri: backendFunction.functionArn,
        });

        let functionRouteProps: CfnRouteProps = {
            apiId: this.apiId,
            routeKey: '$default',
            target: `integrations/${resourceIntegration.ref}`,
        };
        if (props.authorization) {
            new CfnRoute(this, 'UnauthorizedRoute', {
                ...functionRouteProps,
                routeKey: 'OPTIONS /{proxy+}',
            });
            const cognitoAuth = new CfnAuthorizer(this, 'Authorization', {
                apiId: this.apiId,
                authorizerType: 'JWT',
                identitySource: [
                    '$request.header.Authorization',
                ],
                jwtConfiguration: {
                    issuer: props.authorization.issuer,
                    audience: props.authorization.audience
                },
                name: `${apiName}-auth`,
            });
            functionRouteProps = {
                ...functionRouteProps,
                authorizationScopes: props.authorization.scopes,
                authorizationType: 'JWT',
                authorizerId: cognitoAuth.ref,
            };
        }
        const resourceDefaultRoute = new CfnRoute(this, 'DefaultRoute', functionRouteProps);
        const resourceStage = new CfnStage(this, 'Deployment', {
            apiId: this.apiId,
            stageName: '$default',
            autoDeploy: true,
        });
        resourceStage.addDependency(resourceDefaultRoute);
        const stack = Stack.of(this);
        backendFunction.addPermission('Invoke', {
            principal: new ServicePrincipal('apigateway.amazonaws.com'),
            action: 'lambda:InvokeFunction',
            sourceArn: stack.formatArn({
                service: 'execute-api',
                resource: this.apiId,
                arnFormat: ArnFormat.SLASH_RESOURCE_NAME,
                resourceName: "*/*"
            })
        });
        this.stageId = resourceStage.ref;
    }

    addDomain(id: string, props: FamtracApiDomainProps): void {
        const domainCreation = new CfnDomainName(this, `${id}Name`, {
            domainName: props.domainName,
            domainNameConfigurations: [
                {
                    certificateArn: props.certificate.certificateArn,
                    endpointType: 'REGIONAL',
                    securityPolicy: 'TLS_1_2',
                },
            ],
        });
        const mappingResource = new CfnApiMapping(this, `${id}Mapping`, {
            apiId: this.apiId,
            domainName: props.domainName,
            stage: this.stageId,
        });
        mappingResource.addDependency(domainCreation);

        new CnameRecord(this, `${id}CNAME`, {
            domainName: domainCreation.attrRegionalDomainName,
            zone: props.hostedZone,
            recordName: props.domainName,
            ttl: Duration.minutes(5),
        });
    }
}