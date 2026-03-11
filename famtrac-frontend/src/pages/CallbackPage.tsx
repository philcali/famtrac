import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Container, Spinner } from 'react-bootstrap';
import { parseTokensFromUrl, storeTokens } from '../auth/tokenService';

/**
 * CallbackPage - Handles OAuth callback from Cognito
 * Parses tokens from URL and stores them (Requirements 1.2, 1.3)
 * Redirects to preserved URL or home page (Requirement 14.6)
 */
export function CallbackPage() {
  const navigate = useNavigate();

  useEffect(() => {
    const handleCallback = async () => {
      try {
        // Parse tokens from URL (hash or query parameters)
        const tokens = parseTokensFromUrl();

        if (tokens) {
          // Store tokens in sessionStorage
          storeTokens(tokens);

          // Get the URL to redirect to (preserved before login)
          const redirectUrl = sessionStorage.getItem('auth_redirect_url') || '/';
          sessionStorage.removeItem('auth_redirect_url');

          // Redirect to the preserved URL
          navigate(redirectUrl, { replace: true });
        } else {
          // No tokens found, redirect to home
          console.error('No tokens found in callback URL');
          navigate('/', { replace: true });
        }
      } catch (error) {
        console.error('Error handling OAuth callback:', error);
        navigate('/', { replace: true });
      }
    };

    handleCallback();
  }, [navigate]);

  return (
    <Container
      className="d-flex flex-column align-items-center justify-content-center"
      style={{ minHeight: '100vh' }}
    >
      <Spinner animation="border" variant="primary" />
      <p className="mt-3 text-muted">Completing sign in...</p>
    </Container>
  );
}
