import { ArnFormat, Duration, Stack } from "aws-cdk-lib";
import { CfnApi, CfnAuthorizer, CfnIntegration, CfnRoute, CfnRouteProps, CfnStage } from "aws-cdk-lib/aws-apigatewayv2";
import { AttributeType, BillingMode, ITable, ProjectionType, Table } from "aws-cdk-lib/aws-dynamodb";
import { Effect, PolicyStatement, ServicePrincipal } from "aws-cdk-lib/aws-iam";
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";

export interface FamtracApiAuthorizationProps {
    readonly issuer: string;
    readonly audience: string[];
    readonly scopes?: string[];
}

export interface IFamtracApi {
    readonly table: ITable;
    readonly apiId: string;
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
                    name: 'owner_id',
                    type: AttributeType.STRING,
                },
                sortKey: {
                    name: 'created_at',
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

        backendFunction.addToRolePolicy(new PolicyStatement({
            effect: Effect.ALLOW,
            actions: [
                'dynamodb:Query',
            ],
            resources: indexes.map(indexName => `${this.table.tableArn}/index/${indexName}`),
        }));

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
    }
}