import { Duration } from "aws-cdk-lib";
import { AttributeType, BillingMode, ITable, ProjectionType, Table } from "aws-cdk-lib/aws-dynamodb";
import { Effect, PolicyStatement } from "aws-cdk-lib/aws-iam";
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";


export interface IFamtracApi {
    readonly table: ITable;
}

export interface FamtracApiProps {
    readonly apiName?: string;
    readonly table?: ITable;
    readonly enableDevelopmentOrigin?: boolean;
    readonly customOrigins?: string[];
    readonly backendCode: Code;
}

export class FamtracApi extends Construct implements IFamtracApi {
    readonly table: ITable;

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
    }
}