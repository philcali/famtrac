interface EnvironmentConfig {
  apiBaseUrl: string;
  cognito: {
    domain: string;
    clientId: string;
    redirectUri: string;
    logoutUri: string;
    scope: string;
  };
}

function validateConfig(): EnvironmentConfig {
  const apiBaseUrl = import.meta.env.VITE_API_BASE_URL;
  const cognitoDomain = import.meta.env.VITE_COGNITO_DOMAIN;
  const cognitoClientId = import.meta.env.VITE_COGNITO_CLIENT_ID;
  const cognitoRedirectUri = import.meta.env.VITE_COGNITO_REDIRECT_URI;
  const cognitoLogoutUri = import.meta.env.VITE_COGNITO_LOGOUT_URI;
  const cognitoScope = import.meta.env.VITE_COGNITO_SCOPE;

  const missingVars: string[] = [];

  if (!apiBaseUrl) missingVars.push('VITE_API_BASE_URL');
  if (!cognitoDomain) missingVars.push('VITE_COGNITO_DOMAIN');
  if (!cognitoClientId) missingVars.push('VITE_COGNITO_CLIENT_ID');
  if (!cognitoRedirectUri) missingVars.push('VITE_COGNITO_REDIRECT_URI');
  if (!cognitoLogoutUri) missingVars.push('VITE_COGNITO_LOGOUT_URI');
  if (!cognitoScope) missingVars.push('VITE_COGNITO_SCOPE');

  if (missingVars.length > 0) {
    throw new Error(
      `Missing required environment variables: ${missingVars.join(', ')}. ` +
        'Please check your .env file and ensure all required variables are set.'
    );
  }

  return {
    apiBaseUrl,
    cognito: {
      domain: cognitoDomain,
      clientId: cognitoClientId,
      redirectUri: cognitoRedirectUri,
      logoutUri: cognitoLogoutUri,
      scope: cognitoScope,
    },
  };
}

export const config = validateConfig();
