import { config } from './environment';
import { generateCodeVerifier, storeCodeVerifier, deriveCodeChallenge } from '../auth/tokenService';

export interface CognitoConfig {
  domain: string;
  clientId: string;
  redirectUri: string;
  logoutUri: string;
  scope: string;
}

export function getCognitoConfig(): CognitoConfig {
  return config.cognito;
}

export async function buildLoginUrl(): Promise<string> {
  const cognitoConfig = getCognitoConfig();

  const codeVerifier = generateCodeVerifier();
  storeCodeVerifier(codeVerifier);
  const codeChallenge = await deriveCodeChallenge(codeVerifier);

  const params = new URLSearchParams({
    client_id: cognitoConfig.clientId,
    response_type: 'code',
    scope: cognitoConfig.scope,
    redirect_uri: cognitoConfig.redirectUri,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  });

  return `https://${cognitoConfig.domain}/oauth2/authorize?${params.toString()}`;
}

export function buildLogoutUrl(): string {
  const cognitoConfig = getCognitoConfig();
  const params = new URLSearchParams({
    client_id: cognitoConfig.clientId,
    logout_uri: cognitoConfig.logoutUri,
  });

  return `https://${cognitoConfig.domain}/logout?${params.toString()}`;
}
