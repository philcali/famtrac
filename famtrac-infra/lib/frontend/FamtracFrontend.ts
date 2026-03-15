import { Duration } from "aws-cdk-lib";
import { ICertificate } from "aws-cdk-lib/aws-certificatemanager";
import { AllowedMethods, Distribution, HttpVersion, IDistribution, PriceClass, SecurityPolicyProtocol, ViewerProtocolPolicy } from "aws-cdk-lib/aws-cloudfront";
import { S3BucketOrigin } from "aws-cdk-lib/aws-cloudfront-origins";
import { CnameRecord, IHostedZone } from "aws-cdk-lib/aws-route53";
import { BlockPublicAccess, Bucket, BucketEncryption, HttpMethods, IBucket } from "aws-cdk-lib/aws-s3";
import { BucketDeployment, ISource } from "aws-cdk-lib/aws-s3-deployment";
import { Construct } from "constructs";

export interface FamtracFrontendProps {
    readonly storage?: IBucket;
    readonly distribution?: IDistribution;
    readonly certificate?: ICertificate;
    readonly domainNames?: string[];
    readonly hostedZone?: IHostedZone;
    readonly bucketName?: string;
    readonly sources: ISource[];
}

export interface IFamtracFrontend {
    readonly storage: IBucket;
    readonly distribution: IDistribution;
}

export class FamtracFrontend extends Construct implements IFamtracFrontend {
    readonly storage: IBucket;
    readonly distribution: IDistribution;

    constructor(scope: Construct, id: string, props: FamtracFrontendProps) {
        super(scope, id);

        if (props.storage && props.bucketName) {
            throw new Error('Cannot provide both a bucket and a bucket name');
        }

        if (props.distribution && (props.certificate || props.domainNames)) {
            throw new Error('Cannot specify both a distribution and properties for a managed distribution (certificate or domainNames)');
        }

        this.storage = props.storage ?? new Bucket(this, 'Storage', {
            bucketName: props.bucketName,
            blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
            publicReadAccess: false,
            encryption: BucketEncryption.S3_MANAGED,
            cors: [
                {
                    allowedOrigins: [ "*" ],
                    allowedMethods: [
                        HttpMethods.GET,
                        HttpMethods.HEAD,
                    ],
                },
            ],
        });

        this.distribution = props.distribution ?? new Distribution(this, 'Distribution', {
            defaultBehavior: {
                origin: S3BucketOrigin.withOriginAccessControl(this.storage),
                allowedMethods: AllowedMethods.ALLOW_GET_HEAD_OPTIONS,
                viewerProtocolPolicy: ViewerProtocolPolicy.HTTPS_ONLY,
            },
            certificate: props.certificate,
            domainNames: props.domainNames,
            priceClass: PriceClass.PRICE_CLASS_100,
            enabled: true,
            minimumProtocolVersion: SecurityPolicyProtocol.TLS_V1_3_2025,
            httpVersion: HttpVersion.HTTP2,
            errorResponses: [403, 404].map(code => {
                return {
                    httpStatus: code,
                    responseHttpStatus: 200,
                    responsePagePath: '/index.html',
                    ttl: Duration.minutes(5),
                };
            }),
        });

        if (props.hostedZone && props.domainNames) {
            let zone = props.hostedZone;
            props.domainNames.forEach((domainName, index) => {
                new CnameRecord(this, `DistributionCNAME${index}`, {
                    domainName: this.distribution.distributionDomainName,
                    zone,
                    recordName: domainName,
                    ttl: Duration.minutes(5),
                });
            });
        }

        new BucketDeployment(this, 'Deployment', {
            sources: props.sources,
            destinationBucket: this.storage,
            distribution: this.distribution,
        });
    }
}