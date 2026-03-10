import { ArnFormat, AssetOptions, Stack, StackProps } from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { FamtracApi, FamtracApiAuthorizationProps, IFamtracApi } from './backend/FamtracApi';
import { AssetCode } from 'aws-cdk-lib/aws-lambda';
import { FamtracAuthorization, IFamtracAuthorization } from './auth/FamtracAuthorization';
import { Certificate, ICertificate } from 'aws-cdk-lib/aws-certificatemanager';
import { HostedZone, IHostedZone } from 'aws-cdk-lib/aws-route53';
import * as child from 'child_process';
import * as p from 'path';


interface LocalModuleAssetOptions extends AssetOptions {
    readonly buildCommand: string;
    readonly buildOutput: string;
}

class LocalModuleAsset extends AssetCode {
    constructor(path: string, options: LocalModuleAssetOptions) {
        child.execSync(options.buildCommand, {
            cwd: path,
        });
        super(p.join(path, options.buildOutput), {
            ...options,
        });
    }
}

interface IFamtracConfiguration {
    readonly certificate?: ICertificate;
    readonly hostedZone?: IHostedZone;
    readonly enableCustomDomain: boolean;
    readonly domainName: string;
}

class FamtracConfiguration extends Construct implements IFamtracConfiguration {
    readonly certificate?: ICertificate;
    readonly domainName: string;
    readonly hostedZone?: IHostedZone;
    readonly enableCustomDomain: boolean;

    constructor(scope: Construct, id: string) {
        super(scope, id);
        this.enableCustomDomain = this.node.tryGetContext('famtrac/enableCustomDomain') ?? false;
        if (this.enableCustomDomain) {
            const stack = Stack.of(this);
            const certificateId = this.node.tryGetContext('famtrac/certificateId') ?? '';
            this.certificate = Certificate.fromCertificateArn(this, 'Certificate', stack.formatArn({
                service: 'acm',
                resource: 'certificate',
                resourceName: certificateId,
                arnFormat: ArnFormat.SLASH_RESOURCE_NAME,
            }));

            this.domainName = this.node.tryGetContext('famtrac/customDomain') ?? '';
            const hostedZoneId = this.node.tryGetContext('famtrac/hostedZoneId') ?? '';
            this.hostedZone = HostedZone.fromHostedZoneAttributes(this, 'HostedZone', {
                hostedZoneId,
                zoneName: this.domainName,
            });
        }
    }
}

interface FamtracAuthorizationStackProps extends StackProps {
    readonly config: FamtracConfiguration;
}

class FamtracAuthorizationStack extends Stack {
    readonly auth: IFamtracAuthorization;

    constructor(scope: Construct, id: string, props?: FamtracAuthorizationStackProps) {
        super(scope, id, props);
        let auth = new FamtracAuthorization(this, 'Authorization', {
            poolName: 'famtrac-auth',
            enableDevelopmentOrigin: true,
            customOrigins: [
                `https://app.${props?.config.domainName}`
            ]
        });
        if (props?.config.enableCustomDomain && props.config.certificate && props.config.hostedZone) {
            auth.addDomain('CustomDomain', {
                certificate: props.config.certificate,
                hostedZone: props.config.hostedZone,
                domainName: `auth.${props.config.domainName}`,
                createARecord: true,
            });
        }
        this.auth = auth;
    }
}

interface FamtracBackendStackProps extends StackProps {
    readonly config: FamtracConfiguration;
    readonly authorization?: IFamtracAuthorization;
}

class FamtracBackendStack extends Stack {
    readonly api: IFamtracApi;

    constructor(scope: Construct, id: string, props?: FamtracBackendStackProps) {
        super(scope, id, props);

        let authorization: FamtracApiAuthorizationProps | undefined;
        if (props?.authorization) {
            authorization = {
                issuer: props.authorization.userPool.userPoolProviderUrl,
                audience: [
                    props.authorization.userPoolClient.userPoolClientId
                ],
            };
        }
        let api = new FamtracApi(this, 'Api', {
            apiName: 'famtrac-api',
            enableDevelopmentOrigin: true,
            authorization,
            customOrigins: [
                `https://app.${props?.config.domainName}`,
            ],
            backendCode: new LocalModuleAsset(p.join(__dirname, '..', '..', 'famtrac-backend'), {
                buildCommand: './dev.make-zip.sh',
                buildOutput: 'build_famtrac_function.zip',
            }),
        });
        if (props?.config.enableCustomDomain && props.config.certificate && props.config.hostedZone) {
            api.addDomain('CustomDomain', {
                certificate: props.config.certificate,
                hostedZone: props.config.hostedZone,
                domainName: `api.${props.config.domainName}`,
            })
        }
        this.api = api;
    }
}

export class FamtracInfraStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props);

        const config = new FamtracConfiguration(this, 'ImportedConfig');

        const authStack = new FamtracAuthorizationStack(this, 'AuthStack', {
            stackName: 'FamtracAuthorizationStack',
            config,
            
        });

        new FamtracBackendStack(this, 'BackendStack', {
            stackName: 'FamtracBackendStack',
            authorization: authStack.auth,
            config,
        });
    }
}
