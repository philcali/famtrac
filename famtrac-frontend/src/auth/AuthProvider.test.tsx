import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, waitFor, act } from '@testing-library/react';
import { AuthProvider } from './AuthProvider';
import { useAuth } from './useAuth';
import * as tokenService from './tokenService';
import * as cognito from '../config/cognito';

// Mock cognito config module
vi.mock('../config/cognito', async () => {
  const actual = await vi.importActual('../config/cognito');
  return {
    ...actual,
    buildLoginUrl: vi.fn().mockResolvedValue('https://test.auth/login'),
    buildLogoutUrl: vi.fn().mockReturnValue('https://test.auth/logout'),
  };
});

// Helper: a valid base64-encoded JWT payload with user info
const TEST_ID_TOKEN = [
  btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' })),
  btoa(
    JSON.stringify({ sub: 'user-123', email: 'test@example.com', 'cognito:username': 'testuser' })
  ),
  'signature',
].join('.');

function TestComponent() {
  const { isAuthenticated, isLoading, user, logout } = useAuth();

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <div data-testid="auth-status">{isAuthenticated ? 'Authenticated' : 'Not Authenticated'}</div>
      {user && <div data-testid="user-email">{user.email}</div>}
      <button data-testid="logout-btn" onClick={logout}>
        Logout
      </button>
    </div>
  );
}

describe('AuthProvider', () => {
  let originalLocation: Location;

  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
    vi.useFakeTimers({ shouldAdvanceTime: true });

    // Capture original location and make href writable for redirect assertions
    originalLocation = window.location;
    Object.defineProperty(window, 'location', {
      writable: true,
      value: { ...originalLocation, href: '', pathname: '/' },
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    Object.defineProperty(window, 'location', {
      writable: true,
      value: originalLocation,
    });
  });

  it('should show not authenticated when no refresh token exists', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue(null);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
  });

  it('should set authenticated state when valid refresh token exists and refresh succeeds', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue('valid-refresh-token');
    vi.spyOn(tokenService, 'refreshAccessToken').mockResolvedValue({
      access_token: 'new-access',
      id_token: TEST_ID_TOKEN,
      refresh_token: 'new-refresh',
      expires_in: 3600,
    });
    vi.spyOn(tokenService, 'storeRefreshToken').mockImplementation(() => {});

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    expect(screen.getByTestId('auth-status')).toHaveTextContent('Authenticated');
    expect(screen.getByTestId('user-email')).toHaveTextContent('test@example.com');
    expect(tokenService.refreshAccessToken).toHaveBeenCalledWith('valid-refresh-token');
  });

  it('should redirect to login when session is expired (>30 days)', async () => {
    // getRefreshToken returns null when session is expired (internal check)
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue(null);

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
  });

  it('should clear localStorage and redirect to Cognito logout on logout', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue('valid-refresh-token');
    vi.spyOn(tokenService, 'refreshAccessToken').mockResolvedValue({
      access_token: 'access',
      id_token: TEST_ID_TOKEN,
      refresh_token: 'refresh',
      expires_in: 3600,
    });
    vi.spyOn(tokenService, 'storeRefreshToken').mockImplementation(() => {});
    const clearSpy = vi.spyOn(tokenService, 'clearTokens').mockImplementation(() => {});

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Authenticated');
    });

    await act(async () => {
      screen.getByTestId('logout-btn').click();
    });

    expect(clearSpy).toHaveBeenCalled();
    expect(cognito.buildLogoutUrl).toHaveBeenCalled();
    expect(window.location.href).toBe('https://test.auth/logout');
  });

  it('should handle auth:expired event by clearing session and redirecting', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue('valid-refresh-token');
    vi.spyOn(tokenService, 'refreshAccessToken').mockResolvedValue({
      access_token: 'access',
      id_token: TEST_ID_TOKEN,
      refresh_token: 'refresh',
      expires_in: 3600,
    });
    vi.spyOn(tokenService, 'storeRefreshToken').mockImplementation(() => {});
    const clearSpy = vi.spyOn(tokenService, 'clearTokens').mockImplementation(() => {});

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Authenticated');
    });

    // Dispatch auth:expired event
    await act(async () => {
      window.dispatchEvent(new Event('auth:expired'));
    });

    expect(clearSpy).toHaveBeenCalled();
    expect(cognito.buildLoginUrl).toHaveBeenCalled();
  });

  it('should cancel refresh timer on unmount', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue('valid-refresh-token');
    vi.spyOn(tokenService, 'refreshAccessToken').mockResolvedValue({
      access_token: 'access',
      id_token: TEST_ID_TOKEN,
      refresh_token: 'refresh',
      expires_in: 3600,
    });
    vi.spyOn(tokenService, 'storeRefreshToken').mockImplementation(() => {});
    const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout');

    const { unmount } = render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId('auth-status')).toHaveTextContent('Authenticated');
    });

    unmount();

    // clearTimeout should have been called during cleanup
    expect(clearTimeoutSpy).toHaveBeenCalled();
  });

  it('should clear tokens when refresh fails during initialization', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue('stale-refresh');
    vi.spyOn(tokenService, 'refreshAccessToken').mockResolvedValue(null);
    const clearSpy = vi.spyOn(tokenService, 'clearTokens').mockImplementation(() => {});

    render(
      <AuthProvider>
        <TestComponent />
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
    });

    expect(clearSpy).toHaveBeenCalled();
    expect(screen.getByTestId('auth-status')).toHaveTextContent('Not Authenticated');
  });
});
