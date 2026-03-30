import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { ProtectedRoute } from './ProtectedRoute';
import { AuthProvider } from './AuthProvider';
import * as tokenService from './tokenService';
import * as cognito from '../config/cognito';

vi.mock('../config/cognito', async () => {
  const actual = await vi.importActual('../config/cognito');
  return {
    ...actual,
    buildLoginUrl: vi.fn().mockResolvedValue('https://test.auth/login'),
    buildLogoutUrl: vi.fn().mockReturnValue('https://test.auth/logout'),
  };
});

const TEST_ID_TOKEN = [
  btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' })),
  btoa(JSON.stringify({ sub: '1234567890', email: 'test@example.com' })),
  'abc123',
].join('.');

const originalLocation = window.location;

describe('ProtectedRoute', () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
    vi.clearAllMocks();

    Object.defineProperty(window, 'location', {
      writable: true,
      value: { ...originalLocation, href: '', pathname: '/' },
    });
  });

  afterEach(() => {
    Object.defineProperty(window, 'location', {
      writable: true,
      value: originalLocation,
    });
  });

  it('should render children when authenticated', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue('valid-refresh');
    vi.spyOn(tokenService, 'refreshAccessToken').mockResolvedValue({
      access_token: 'test_access',
      id_token: TEST_ID_TOKEN,
      refresh_token: 'valid-refresh',
      expires_in: 3600,
    });
    vi.spyOn(tokenService, 'storeRefreshToken').mockImplementation(() => {});

    render(
      <AuthProvider>
        <ProtectedRoute>
          <div>Protected Content</div>
        </ProtectedRoute>
      </AuthProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Protected Content')).toBeInTheDocument();
    });
  });

  it('should redirect to login when not authenticated', async () => {
    vi.spyOn(tokenService, 'getRefreshToken').mockReturnValue(null);

    render(
      <AuthProvider>
        <ProtectedRoute>
          <div>Protected Content</div>
        </ProtectedRoute>
      </AuthProvider>
    );

    await waitFor(() => {
      expect(cognito.buildLoginUrl).toHaveBeenCalled();
    });
  });
});
