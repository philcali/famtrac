export interface CognitoTokens {
  access_token: string;
  id_token: string;
  refresh_token?: string;
  expires_in: number;
  token_type?: string;
}

// --- Constants ---
export const AUTH_KEY_PREFIX = 'famtrac_auth_';
const REFRESH_TOKEN_KEY = `${AUTH_KEY_PREFIX}refresh_token`;
const TOKEN_EXPIRY_KEY = `${AUTH_KEY_PREFIX}token_expiry`;
const SESSION_TIMESTAMP_KEY = `${AUTH_KEY_PREFIX}session_timestamp`;
const CODE_VERIFIER_KEY = `${AUTH_KEY_PREFIX}code_verifier`;

export const SESSION_LIFETIME_MS = 30 * 24 * 60 * 60 * 1000; // 30 days

// --- PKCE Helpers ---

/**
 * Generate a 128-character random string for use as a PKCE code verifier.
 * Uses crypto.getRandomValues() for cryptographic randomness.
 */
export function generateCodeVerifier(): string {
  const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
  const randomValues = crypto.getRandomValues(new Uint8Array(128));
  let verifier = '';
  for (let i = 0; i < 128; i++) {
    verifier += charset[randomValues[i] % charset.length];
  }
  return verifier;
}

/**
 * Derive a PKCE code challenge from a code verifier using SHA-256 + base64url encoding.
 */
export async function deriveCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  const bytes = new Uint8Array(digest);
  // base64url encode
  let base64 = btoa(String.fromCharCode(...bytes));
  base64 = base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  return base64;
}

/**
 * Store the PKCE code verifier in localStorage for use during the callback token exchange.
 */
export function storeCodeVerifier(verifier: string): void {
  localStorage.setItem(CODE_VERIFIER_KEY, verifier);
}

/**
 * Retrieve and remove the stored PKCE code verifier from localStorage.
 * Returns null if no verifier is stored.
 */
export function getAndClearCodeVerifier(): string | null {
  const verifier = localStorage.getItem(CODE_VERIFIER_KEY);
  if (verifier) {
    localStorage.removeItem(CODE_VERIFIER_KEY);
  }
  return verifier;
}

// --- Token Exchange ---

/**
 * Exchange an authorization code for tokens by POSTing to the Cognito /oauth2/token endpoint.
 * Uses the stored PKCE code_verifier for the exchange.
 */
export async function exchangeCodeForTokens(code: string): Promise<CognitoTokens | null> {
  const cognitoConfig = await import('../config/cognito').then((m) => m.getCognitoConfig());
  const codeVerifier = getAndClearCodeVerifier();

  if (!codeVerifier) {
    console.error('No code verifier found for token exchange');
    return null;
  }

  try {
    const response = await fetch(`https://${cognitoConfig.domain}/oauth2/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        client_id: cognitoConfig.clientId,
        code,
        code_verifier: codeVerifier,
        redirect_uri: cognitoConfig.redirectUri,
      }),
    });

    if (!response.ok) {
      console.error('Token exchange failed:', response.status);
      return null;
    }

    const data = await response.json();
    return {
      access_token: data.access_token,
      id_token: data.id_token,
      refresh_token: data.refresh_token,
      expires_in: data.expires_in,
      token_type: data.token_type,
    };
  } catch (error) {
    console.error('Token exchange error:', error);
    return null;
  }
}

// --- Persistent Storage (refresh token + metadata only) ---

/**
 * Store the refresh token and metadata in localStorage.
 * Session timestamp is only written on the first call (preserved across refreshes).
 */
export function storeRefreshToken(refreshToken: string, expiresIn: number): void {
  localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
  const expiryTime = Date.now() + expiresIn * 1000;
  localStorage.setItem(TOKEN_EXPIRY_KEY, expiryTime.toString());

  // Only set session timestamp if not already present (preserve across refreshes)
  if (!localStorage.getItem(SESSION_TIMESTAMP_KEY)) {
    localStorage.setItem(SESSION_TIMESTAMP_KEY, Date.now().toString());
  }
}

/**
 * Get the stored refresh token, checking the 30-day session lifetime first.
 * Returns null and clears all auth keys if the session has expired.
 */
export function getRefreshToken(): string | null {
  if (isSessionExpired()) {
    clearTokens();
    return null;
  }
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

/**
 * Get the stored token expiry timestamp.
 */
export function getTokenExpiry(): number | null {
  const expiry = localStorage.getItem(TOKEN_EXPIRY_KEY);
  if (!expiry) {
    return null;
  }
  return parseInt(expiry, 10);
}

/**
 * Check if the 30-day session lifetime has been exceeded.
 */
export function isSessionExpired(): boolean {
  const timestamp = localStorage.getItem(SESSION_TIMESTAMP_KEY);
  if (!timestamp) {
    return true;
  }
  const sessionStart = parseInt(timestamp, 10);
  return Date.now() - sessionStart >= SESSION_LIFETIME_MS;
}

// --- Refresh ---

/**
 * Refresh the access token using a refresh token.
 * POSTs to the Cognito /oauth2/token endpoint with grant_type=refresh_token.
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
      refresh_token: refreshToken, // Cognito does not return refresh token on refresh grant
      expires_in: data.expires_in,
      token_type: data.token_type,
    };
  } catch (error) {
    console.error('Token refresh error:', error);
    return null;
  }
}

// --- Cleanup ---

/**
 * Clear all authentication-related keys from localStorage.
 * Removes any key that starts with the AUTH_KEY_PREFIX, except the
 * code verifier which is a transient PKCE artifact needed by the
 * callback page during the authorization code exchange.
 */
export function clearTokens(): void {
  const keysToRemove: string[] = [];
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && key.startsWith(AUTH_KEY_PREFIX) && key !== CODE_VERIFIER_KEY) {
      keysToRemove.push(key);
    }
  }
  for (const key of keysToRemove) {
    localStorage.removeItem(key);
  }
}
