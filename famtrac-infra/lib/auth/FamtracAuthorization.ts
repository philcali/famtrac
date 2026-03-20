import { Duration } from "aws-cdk-lib";
import { ICertificate } from "aws-cdk-lib/aws-certificatemanager";
import { AccountRecovery, ClientAttributes, Mfa, OAuthScope, UserPool, UserPoolClient, UserPoolDomain, UserPoolEmail } from "aws-cdk-lib/aws-cognito";
import { ARecord, CnameRecord, IHostedZone, RecordTarget } from "aws-cdk-lib/aws-route53";
import { Construct } from "constructs";

export interface FamtracAuthorizationDomainProps {
    readonly certificate: ICertificate;
    readonly hostedZone: IHostedZone;
    readonly domainName: string;
    readonly createARecord?: boolean;
}

export interface FamtractAuthorizationProps {
    readonly poolName?: string;
    readonly enableDevelopmentOrigin?: boolean;
    readonly customOrigins?: string[];
}

export interface IFamtracAuthorization {
    readonly userPool: UserPool;
    readonly userPoolClient: UserPoolClient;
}

export class FamtracAuthorization extends Construct implements IFamtracAuthorization {
    readonly userPool: UserPool;
    readonly userPoolClient: UserPoolClient;

    constructor(scope: Construct, id: string, props?: FamtractAuthorizationProps) {
        super(scope, id);

        const userPoolName = props?.poolName ?? 'famtrac-user-pool';
        const userPool = new UserPool(this, 'Users', {
            userPoolName,
            email: UserPoolEmail.withCognito('noreply@verificationemail.com'),
            selfSignUpEnabled: false,
            mfa: Mfa.REQUIRED,
            accountRecovery: AccountRecovery.EMAIL_ONLY,
            enableSmsRole: false,
            mfaSecondFactor: {
                otp: true,
                sms: false,
            },
            passwordPolicy: {
                minLength: 12,
                requireSymbols: true,
                requireDigits: true,
            },
            signInCaseSensitive: false,
            signInAliases: {
                username: true,
                email: true,
            },
            autoVerify: {
                email: true,
            },
            keepOriginal: {
                email: true,
            },
        });

        const writeAttributes = new ClientAttributes().withStandardAttributes({
            fullname: true,
            email: true,
        });
        const readAttributes = writeAttributes.withStandardAttributes({
            email: true,
            emailVerified: true,
        });

        let redirectOrigins = [];
        if (props?.enableDevelopmentOrigin === true) {
            redirectOrigins.push('http://localhost:5173');
        }
        props?.customOrigins?.forEach(origin => redirectOrigins.push(origin));
        const userPoolClient = userPool.addClient('Client', {
            generateSecret: true,
            authFlows: {
                userSrp: true,
                userPassword: true,
            },
            enableTokenRevocation: true,
            accessTokenValidity: Duration.days(1),
            refreshTokenValidity: Duration.days(365),
            idTokenValidity: Duration.days(1),
            readAttributes,
            writeAttributes,
            userPoolClientName: `${userPoolName}-client`,
            oAuth: {
                flows: {
                    authorizationCodeGrant: true,
                    implicitCodeGrant: true,   
                },
                scopes: [
                    OAuthScope.OPENID,
                    OAuthScope.EMAIL,
                    OAuthScope.PROFILE
                ],
                callbackUrls: redirectOrigins.map(origin => `${origin}/login`),
                logoutUrls: redirectOrigins.map(origin => `${origin}/logout`),
            }
        });
        this.userPool = userPool;
        this.userPoolClient = userPoolClient;
    }

    addDomain(id: string, props: FamtracAuthorizationDomainProps): void{
        let arecord;
        if (props.createARecord === true) {
            arecord = new ARecord(this, `${id}ARecord`, {
                zone: props.hostedZone,
                target: RecordTarget.fromIpAddresses('198.51.100.1'),
            });
        }

        const customAuthDomain = this.userPool.addDomain(`${id}Pool`, {
            customDomain: {
                certificate: props.certificate,
                domainName: props.domainName,
            },
        });

        if (arecord) {
            customAuthDomain.node.addDependency(arecord);
        } 

        new CnameRecord(this, `${id}CNAMERecord`, {
            domainName: customAuthDomain.cloudFrontEndpoint,
            zone: props.hostedZone,
            recordName: props.domainName,
            ttl: Duration.minutes(5),
        });
    }
}