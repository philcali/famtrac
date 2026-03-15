import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { CallbackPage } from './CallbackPage';
import * as tokenService from '../auth/tokenService';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe('CallbackPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    sessionStorage.clear();
    window.location.hash = '';
  });

  it('should display a loading spinner while processing', () => {
    vi.spyOn(tokenService, 'parseTokensFromUrl').mockReturnValue(null);

    render(
      <MemoryRouter>
        <CallbackPage />
      </MemoryRouter>
    );

    expect(screen.getByText('Completing sign in...')).toBeInTheDocument();
    expect(document.querySelector('.spinner-border')).toBeInTheDocument();
  });

  it('should parse tokens from URL and store them (Req 1.3)', async () => {
    const mockTokens: tokenService.CognitoTokens = {
      access_token: 'test_access_token',
      id_token: 'test_id_token',
      expires_in: 3600,
    };

    vi.spyOn(tokenService, 'parseTokensFromUrl').mockReturnValue(mockTokens);
    const storeTokensSpy = vi.spyOn(tokenService, 'storeTokens');

    render(
      <MemoryRouter>
        <CallbackPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(storeTokensSpy).toHaveBeenCalledWith(mockTokens);
    });
  });

  it('should redirect to preserved URL after storing tokens (Req 14.6)', async () => {
    const mockTokens: tokenService.CognitoTokens = {
      access_token: 'test_access_token',
      id_token: 'test_id_token',
      expires_in: 3600,
    };

    vi.spyOn(tokenService, 'parseTokensFromUrl').mockReturnValue(mockTokens);
    vi.spyOn(tokenService, 'storeTokens');
    sessionStorage.setItem('auth_redirect_url', '/families/123');

    render(
      <MemoryRouter>
        <CallbackPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/families/123', { replace: true });
    });

    // Should clean up the stored redirect URL
    expect(sessionStorage.getItem('auth_redirect_url')).toBeNull();
  });

  it('should redirect to home when no preserved URL exists', async () => {
    const mockTokens: tokenService.CognitoTokens = {
      access_token: 'test_access_token',
      id_token: 'test_id_token',
      expires_in: 3600,
    };

    vi.spyOn(tokenService, 'parseTokensFromUrl').mockReturnValue(mockTokens);
    vi.spyOn(tokenService, 'storeTokens');

    render(
      <MemoryRouter>
        <CallbackPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/', { replace: true });
    });
  });

  it('should redirect to home when no tokens found in URL', async () => {
    vi.spyOn(tokenService, 'parseTokensFromUrl').mockReturnValue(null);
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(
      <MemoryRouter>
        <CallbackPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/', { replace: true });
    });

    expect(consoleSpy).toHaveBeenCalledWith('No tokens found in callback URL');
    consoleSpy.mockRestore();
  });

  it('should redirect to home on error during callback handling', async () => {
    vi.spyOn(tokenService, 'parseTokensFromUrl').mockImplementation(() => {
      throw new Error('Parse error');
    });
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    render(
      <MemoryRouter>
        <CallbackPage />
      </MemoryRouter>
    );

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/', { replace: true });
    });

    expect(consoleSpy).toHaveBeenCalledWith('Error handling OAuth callback:', expect.any(Error));
    consoleSpy.mockRestore();
  });
});
