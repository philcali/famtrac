import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { AuthProvider } from './AuthProvider';
import { useAuth } from './useAuth';
import * as tokenService from './tokenService';

// Test component that uses the auth hook
function TestComponent() {
  const { isAuthenticated, isLoading, user } = useAuth();

  if (isLoading) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <div data-testid="auth-status">
        {isAuthenticated ? 'Authenticated' : 'Not Authenticated'}
      </div>
      {user && <div data-testid="user-email">{user.email}</div>}
    </div>
  );
}

describe('AuthProvider', () => {
  beforeEach(() => {
    sessionStorage.clear();
    window.location.hash = '';
    vi.clearAllMocks();
  });

  it('should render children when not authenticated', async () => {
    vi.spyOn(tokenService, 'getAccessToken').mockReturnValue(null);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    expect(screen.getByTestId('auth-status')).toHaveTextContent(
      'Not Authenticated'
    );
  });

  it('should set authenticated state when valid token exists', async () => {
    const mockIdToken =
      'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiZW1haWwiOiJ0ZXN0QGV4YW1wbGUuY29tIn0.abc123';

    vi.spyOn(tokenService, 'getAccessToken').mockReturnValue('test_token');
    vi.spyOn(tokenService, 'getIdToken').mockReturnValue(mockIdToken);
    vi.spyOn(tokenService, 'isTokenExpired').mockReturnValue(false);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    expect(screen.getByTestId('auth-status')).toHaveTextContent(
      'Authenticated'
    );
    expect(screen.getByTestId('user-email')).toHaveTextContent(
      'test@example.com'
    );
  });

  it('should handle OAuth callback with tokens in URL', async () => {
    window.location.hash =
      '#access_token=test_access&id_token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiZW1haWwiOiJ0ZXN0QGV4YW1wbGUuY29tIn0.abc123&expires_in=3600';

    const storeTokensSpy = vi.spyOn(tokenService, 'storeTokens');

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    expect(storeTokensSpy).toHaveBeenCalled();
    expect(screen.getByTestId('auth-status')).toHaveTextContent(
      'Authenticated'
    );
  });
});
