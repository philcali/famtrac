import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { ProtectedRoute } from './ProtectedRoute';
import { AuthProvider } from './AuthProvider';
import * as tokenService from './tokenService';

describe('ProtectedRoute', () => {
  beforeEach(() => {
    sessionStorage.clear();
    vi.clearAllMocks();
    // Mock window.location properly
    Object.defineProperty(window, 'location', {
      value: {
        href: '',
        pathname: '/',
        hash: '',
      },
      writable: true,
    });
  });

  it('should render children when authenticated', async () => {
    const mockIdToken =
      'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiZW1haWwiOiJ0ZXN0QGV4YW1wbGUuY29tIn0.abc123';

    vi.spyOn(tokenService, 'getAccessToken').mockReturnValue('test_token');
    vi.spyOn(tokenService, 'getIdToken').mockReturnValue(mockIdToken);
    vi.spyOn(tokenService, 'isTokenExpired').mockReturnValue(false);

    render(
      <AuthProvider>
        <ProtectedRoute>
          <div>Protected Content</div>
        </ProtectedRoute>
      </AuthProvider>
    );

    // Wait for loading to finish and content to appear
    await waitFor(() => {
      expect(screen.getByText('Protected Content')).toBeInTheDocument();
    });
  });

  it('should redirect to login when not authenticated', async () => {
    vi.spyOn(tokenService, 'getAccessToken').mockReturnValue(null);

    render(
      <AuthProvider>
        <ProtectedRoute>
          <div>Protected Content</div>
        </ProtectedRoute>
      </AuthProvider>
    );

    // Wait for redirect to be triggered
    await waitFor(() => {
      expect(window.location.href).toContain('amazoncognito.com');
    });
  });
});
