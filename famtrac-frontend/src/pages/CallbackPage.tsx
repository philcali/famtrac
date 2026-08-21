import { useEffect, useRef } from 'react';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { exchangeCodeForTokens, storeRefreshToken, clearTokens } from '../auth/tokenService';

/**
 * CallbackPage - Handles OAuth authorization code callback from Cognito.
 * Reads the authorization code from URL query parameters, exchanges it for tokens
 * via the Cognito /oauth2/token endpoint with PKCE, stores the refresh token in
 * localStorage, and redirects to the preserved URL or home page.
 * (Requirements 1.3, 1.4, 1.5)
 */
export function CallbackPage() {
  // Guard against React StrictMode double-invocation of useEffect in dev mode.
  // The code verifier is consumed (read + deleted) on the first run, so a second
  // run would fail with "No code verifier found".
  const exchangeStarted = useRef(false);

  useEffect(() => {
    if (exchangeStarted.current) return;
    exchangeStarted.current = true;

    const handleCallback = async () => {
      try {
        const params = new URLSearchParams(window.location.search);
        const code = params.get('code');

        if (!code) {
          console.error('No authorization code found in callback URL');
          window.location.href = '/';
          return;
        }

        const tokens = await exchangeCodeForTokens(code);

        if (tokens) {
          // Store refresh token and metadata in localStorage
          if (tokens.refresh_token) {
            storeRefreshToken(tokens.refresh_token, tokens.expires_in);
          }

          // Redirect with full page reload so AuthProvider re-initializes
          // and picks up the stored refresh token to obtain fresh in-memory tokens
          const redirectUrl = sessionStorage.getItem('auth_redirect_url') || '/';
          sessionStorage.removeItem('auth_redirect_url');
          window.location.href = redirectUrl;
        } else {
          // Token exchange failed — clear any partial state
          clearTokens();
          window.location.href = '/';
        }
      } catch (error) {
        console.error('Error handling OAuth callback:', error);
        clearTokens();
        window.location.href = '/';
      }
    };

    handleCallback();
  }, []);

  return (
    <div className="flex flex-col items-center justify-center min-h-screen">
      <LoadingSpinner />
      <p className="mt-3 text-muted">Completing sign in...</p>
    </div>
  );
}
