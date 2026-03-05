import { AssetOptions, Stack, StackProps } from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { FamtracApi, IFamtracApi } from './backend/FamtracApi';
import { AssetCode } from 'aws-cdk-lib/aws-lambda';
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

class FamtracBackendStack extends Stack {
    readonly api: IFamtracApi;

    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props);

        let api = new FamtracApi(this, 'Api', {
            apiName: 'famtrac-api',
            enableDevelopmentOrigin: true,
            backendCode: new LocalModuleAsset(p.join(__dirname, '..', '..', 'famtrac-backend'), {
                buildCommand: './dev.make-zip.sh',
                buildOutput: 'build_famtrac_function.zip',
            }),
        });
        this.api = api;
    }
}

export class FamtracInfraStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props);


        new FamtracBackendStack(this, 'BackendStack', {
            stackName: 'FamtracBackendStack',
        });
    }
}
