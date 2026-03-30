import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { CallbackPage } from './CallbackPage';
import * as tokenService from '../auth/tokenService';

// Mock window.location
const originalLocation = window.location;

beforeEach(() => {
  vi.clearAllMocks();
  sessionStorage.clear();
  localStorage.clear();

  // Replace window.location with a writable mock
  Object.defineProperty(window, 'location', {
    writable: true,
    value: { ...originalLocation, href: '', search: '' },
  });
});

afterEach(() => {
  Object.defineProperty(window, 'location', {
    writable: true,
    value: originalLocation,
  });
});

describe('CallbackPage', () => {
  it('should display a loading spinner while processing', () => {
    render(<CallbackPage />);

    expect(screen.getByText('Completing sign in...')).toBeInTheDocument();
    expect(document.querySelector('.spinner-border')).toBeInTheDocument();
  });

  it('should exchange authorization code for tokens and store refresh token (Req 1.3, 1.4)', async () => {
    window.location.search = '?code=test_auth_code';

    const mockTokens: tokenService.CognitoTokens = {
      access_token: 'test_access',
      id_token: 'test_id',
      refresh_token: 'test_refresh',
      expires_in: 3600,
    };

    const exchangeSpy = vi
      .spyOn(tokenService, 'exchangeCodeForTokens')
      .mockResolvedValue(mockTokens);
    const storeSpy = vi.spyOn(tokenService, 'storeRefreshToken');

    render(<CallbackPage />);

    await waitFor(() => {
      expect(exchangeSpy).toHaveBeenCalledWith('test_auth_code');
      expect(storeSpy).toHaveBeenCalledWith('test_refresh', 3600);
      expect(window.location.href).toBe('/');
    });
  });

  it('should redirect to preserved URL after successful exchange', async () => {
    window.location.search = '?code=test_auth_code';
    sessionStorage.setItem('auth_redirect_url', '/families/123');

    const mockTokens: tokenService.CognitoTokens = {
      access_token: 'test_access',
      id_token: 'test_id',
      refresh_token: 'test_refresh',
      expires_in: 3600,
    };

    vi.spyOn(tokenService, 'exchangeCodeForTokens').mockResolvedValue(mockTokens);
    vi.spyOn(tokenService, 'storeRefreshToken');

    render(<CallbackPage />);

    await waitFor(() => {
      expect(window.location.href).toBe('/families/123');
      expect(sessionStorage.getItem('auth_redirect_url')).toBeNull();
    });
  });

  it('should redirect to / when no authorization code is present (Req 1.5)', async () => {
    window.location.search = '';
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(<CallbackPage />);

    await waitFor(() => {
      expect(window.location.href).toBe('/');
    });

    expect(consoleSpy).toHaveBeenCalledWith('No authorization code found in callback URL');
    consoleSpy.mockRestore();
  });

  it('should clear tokens and redirect to / when token exchange fails (Req 1.5)', async () => {
    window.location.search = '?code=bad_code';

    vi.spyOn(tokenService, 'exchangeCodeForTokens').mockResolvedValue(null);
    const clearSpy = vi.spyOn(tokenService, 'clearTokens');

    render(<CallbackPage />);

    await waitFor(() => {
      expect(clearSpy).toHaveBeenCalled();
      expect(window.location.href).toBe('/');
    });
  });

  it('should clear tokens and redirect to / on unexpected error', async () => {
    window.location.search = '?code=test_code';

    vi.spyOn(tokenService, 'exchangeCodeForTokens').mockRejectedValue(new Error('Network error'));
    const clearSpy = vi.spyOn(tokenService, 'clearTokens');
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(<CallbackPage />);

    await waitFor(() => {
      expect(clearSpy).toHaveBeenCalled();
      expect(window.location.href).toBe('/');
    });

    expect(consoleSpy).toHaveBeenCalledWith('Error handling OAuth callback:', expect.any(Error));
    consoleSpy.mockRestore();
  });

  it('should not store refresh token when exchange returns tokens without refresh_token', async () => {
    window.location.search = '?code=test_code';

    const mockTokens: tokenService.CognitoTokens = {
      access_token: 'test_access',
      id_token: 'test_id',
      expires_in: 3600,
      // no refresh_token
    };

    vi.spyOn(tokenService, 'exchangeCodeForTokens').mockResolvedValue(mockTokens);
    const storeSpy = vi.spyOn(tokenService, 'storeRefreshToken');

    render(<CallbackPage />);

    await waitFor(() => {
      expect(window.location.href).toBe('/');
      expect(storeSpy).not.toHaveBeenCalled();
    });
  });
});
