export interface CognitoTokens {
  access_token: string;
  id_token: string;
  refresh_token?: string;
  expires_in: number;
  token_type?: string;
}

const TOKEN_STORAGE_KEY = 'cognito_tokens';
const TOKEN_EXPIRY_KEY = 'token_expiry';

/**
 * Parse tokens from URL hash fragment (OAuth implicit flow)
 * Cognito returns tokens in the URL hash after successful authentication
 */
export function parseTokensFromUrl(): CognitoTokens | null {
  if (!window.location.hash) {
    return null;
  }

  const hash = window.location.hash.substring(1);
  if (!hash) {
    return null;
  }

  const params = new URLSearchParams(hash);
  const accessToken = params.get('access_token');
  const idToken = params.get('id_token');
  const expiresIn = params.get('expires_in');
  const tokenType = params.get('token_type');
  const refreshToken = params.get('refresh_token');

  if (!accessToken || !idToken || !expiresIn) {
    return null;
  }

  return {
    access_token: accessToken,
    id_token: idToken,
    refresh_token: refreshToken || undefined,
    expires_in: parseInt(expiresIn, 10),
    token_type: tokenType || undefined,
  };
}

/**
 * Store tokens in sessionStorage for security
 * sessionStorage is cleared when the browser tab is closed
 */
export function storeTokens(tokens: CognitoTokens): void {
  sessionStorage.setItem(TOKEN_STORAGE_KEY, JSON.stringify(tokens));

  // Calculate and store expiry timestamp
  const expiryTime = Date.now() + tokens.expires_in * 1000;
  sessionStorage.setItem(TOKEN_EXPIRY_KEY, expiryTime.toString());
}

/**
 * Get the access token from storage
 */
export function getAccessToken(): string | null {
  const tokensJson = sessionStorage.getItem(TOKEN_STORAGE_KEY);
  if (!tokensJson) {
    return null;
  }

  try {
    const tokens: CognitoTokens = JSON.parse(tokensJson);
    return tokens.access_token;
  } catch (error) {
    console.error('Failed to parse stored tokens:', error);
    return null;
  }
}

/**
 * Get the ID token from storage
 */
export function getIdToken(): string | null {
  const tokensJson = sessionStorage.getItem(TOKEN_STORAGE_KEY);
  if (!tokensJson) {
    return null;
  }

  try {
    const tokens: CognitoTokens = JSON.parse(tokensJson);
    return tokens.id_token;
  } catch (error) {
    console.error('Failed to parse stored tokens:', error);
    return null;
  }
}

/**
 * Get the refresh token from storage
 */
export function getRefreshToken(): string | null {
  const tokensJson = sessionStorage.getItem(TOKEN_STORAGE_KEY);
  if (!tokensJson) {
    return null;
  }

  try {
    const tokens: CognitoTokens = JSON.parse(tokensJson);
    return tokens.refresh_token || null;
  } catch (error) {
    console.error('Failed to parse stored tokens:', error);
    return null;
  }
}

/**
 * Clear all tokens from storage
 */
export function clearTokens(): void {
  sessionStorage.removeItem(TOKEN_STORAGE_KEY);
  sessionStorage.removeItem(TOKEN_EXPIRY_KEY);
}

/**
 * Check if the current token is expired
 */
export function isTokenExpired(): boolean {
  const expiryTimeStr = sessionStorage.getItem(TOKEN_EXPIRY_KEY);
  if (!expiryTimeStr) {
    return true;
  }

  const expiryTime = parseInt(expiryTimeStr, 10);
  // Add a 60-second buffer to refresh before actual expiry
  return Date.now() >= expiryTime - 60000;
}

/**
 * Refresh the access token using the refresh token
 * Note: This requires the Cognito token endpoint
 */
export async function refreshAccessToken(refreshToken: string): Promise<CognitoTokens | null> {
  const cognitoConfig = await import('../config/cognito').then((m) => m.getCognitoConfig());

  try {
    const response = await fetch(`https://${cognitoConfig.domain}/oauth2/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: cognitoConfig.clientId,
        refresh_token: refreshToken,
      }),
    });

    if (!response.ok) {
      console.error('Token refresh failed:', response.status);
      return null;
    }

    const data = await response.json();
    return {
      access_token: data.access_token,
      id_token: data.id_token,
      refresh_token: refreshToken, // Refresh token is not returned, reuse existing
      expires_in: data.expires_in,
      token_type: data.token_type,
    };
  } catch (error) {
    console.error('Token refresh error:', error);
    return null;
  }
}
