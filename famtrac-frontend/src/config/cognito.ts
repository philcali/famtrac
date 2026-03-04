import { config } from './environment';

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

export function buildLoginUrl(): string {
  const cognitoConfig = getCognitoConfig();
  const params = new URLSearchParams({
    client_id: cognitoConfig.clientId,
    response_type: 'token',
    scope: cognitoConfig.scope,
    redirect_uri: cognitoConfig.redirectUri,
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
