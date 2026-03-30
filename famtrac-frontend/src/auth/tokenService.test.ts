import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import {
  generateCodeVerifier,
  storeCodeVerifier,
  getAndClearCodeVerifier,
  storeRefreshToken,
  getRefreshToken,
  getTokenExpiry,
  isSessionExpired,
  clearTokens,
  AUTH_KEY_PREFIX,
  SESSION_LIFETIME_MS,
} from './tokenService';

describe('tokenService', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('generateCodeVerifier', () => {
    it('should produce a 128-character string', () => {
      const verifier = generateCodeVerifier();
      expect(verifier).toHaveLength(128);
    });

    it('should only contain unreserved URI characters', () => {
      const verifier = generateCodeVerifier();
      // PKCE unreserved characters: [A-Z] [a-z] [0-9] - . _ ~
      expect(verifier).toMatch(/^[A-Za-z0-9\-._~]+$/);
    });

    it('should produce different values on successive calls', () => {
      const a = generateCodeVerifier();
      const b = generateCodeVerifier();
      expect(a).not.toBe(b);
    });
  });

  describe('storeCodeVerifier / getAndClearCodeVerifier', () => {
    it('should store and retrieve the code verifier', () => {
      storeCodeVerifier('test-verifier-123');
      const result = getAndClearCodeVerifier();
      expect(result).toBe('test-verifier-123');
    });

    it('should remove the verifier after retrieval', () => {
      storeCodeVerifier('test-verifier-123');
      getAndClearCodeVerifier();
      const second = getAndClearCodeVerifier();
      expect(second).toBeNull();
    });

    it('should return null when no verifier is stored', () => {
      expect(getAndClearCodeVerifier()).toBeNull();
    });
  });

  describe('storeRefreshToken / getRefreshToken', () => {
    it('should store and retrieve a refresh token', () => {
      storeRefreshToken('my-refresh-token', 3600);
      const token = getRefreshToken();
      expect(token).toBe('my-refresh-token');
    });

    it('should store token expiry as current time + expiresIn * 1000', () => {
      const now = Date.now();
      vi.spyOn(Date, 'now').mockReturnValue(now);

      storeRefreshToken('rt', 7200);
      const expiry = getTokenExpiry();
      expect(expiry).toBe(now + 7200 * 1000);
    });

    it('should set session timestamp on first call only', () => {
      const t1 = 1000000;
      const t2 = 2000000;
      vi.spyOn(Date, 'now')
        .mockReturnValueOnce(t1)
        .mockReturnValueOnce(t1)
        .mockReturnValueOnce(t2)
        .mockReturnValueOnce(t2);

      storeRefreshToken('rt-1', 3600);
      const firstTimestamp = localStorage.getItem(`${AUTH_KEY_PREFIX}session_timestamp`);

      storeRefreshToken('rt-2', 3600);
      const secondTimestamp = localStorage.getItem(`${AUTH_KEY_PREFIX}session_timestamp`);

      expect(firstTimestamp).toBe(t1.toString());
      expect(secondTimestamp).toBe(t1.toString());
    });

    it('should return null when no token is stored', () => {
      // Need a session timestamp so isSessionExpired doesn't clear
      expect(getRefreshToken()).toBeNull();
    });
  });

  describe('isSessionExpired', () => {
    it('should return true when no session timestamp exists', () => {
      expect(isSessionExpired()).toBe(true);
    });

    it('should return false for a session within 30 days', () => {
      const now = Date.now();
      localStorage.setItem(`${AUTH_KEY_PREFIX}session_timestamp`, now.toString());
      expect(isSessionExpired()).toBe(false);
    });

    it('should return true for a session older than 30 days', () => {
      const expired = Date.now() - SESSION_LIFETIME_MS - 1;
      localStorage.setItem(`${AUTH_KEY_PREFIX}session_timestamp`, expired.toString());
      expect(isSessionExpired()).toBe(true);
    });

    it('should return true for a session exactly at 30 days', () => {
      const boundary = Date.now() - SESSION_LIFETIME_MS;
      localStorage.setItem(`${AUTH_KEY_PREFIX}session_timestamp`, boundary.toString());
      expect(isSessionExpired()).toBe(true);
    });
  });

  describe('getRefreshToken with expired session', () => {
    it('should return null and clear tokens when session is expired', () => {
      const expired = Date.now() - SESSION_LIFETIME_MS - 1;
      localStorage.setItem(`${AUTH_KEY_PREFIX}session_timestamp`, expired.toString());
      localStorage.setItem(`${AUTH_KEY_PREFIX}refresh_token`, 'old-token');

      const result = getRefreshToken();
      expect(result).toBeNull();
      // All auth keys should be cleared
      expect(localStorage.getItem(`${AUTH_KEY_PREFIX}refresh_token`)).toBeNull();
      expect(localStorage.getItem(`${AUTH_KEY_PREFIX}session_timestamp`)).toBeNull();
    });
  });

  describe('clearTokens', () => {
    it('should remove all auth-prefixed keys except the code verifier', () => {
      localStorage.setItem(`${AUTH_KEY_PREFIX}refresh_token`, 'rt');
      localStorage.setItem(`${AUTH_KEY_PREFIX}token_expiry`, '123');
      localStorage.setItem(`${AUTH_KEY_PREFIX}session_timestamp`, '456');
      localStorage.setItem(`${AUTH_KEY_PREFIX}code_verifier`, 'cv');

      clearTokens();

      expect(localStorage.getItem(`${AUTH_KEY_PREFIX}refresh_token`)).toBeNull();
      expect(localStorage.getItem(`${AUTH_KEY_PREFIX}token_expiry`)).toBeNull();
      expect(localStorage.getItem(`${AUTH_KEY_PREFIX}session_timestamp`)).toBeNull();
      // Code verifier is preserved for the PKCE callback flow
      expect(localStorage.getItem(`${AUTH_KEY_PREFIX}code_verifier`)).toBe('cv');
    });

    it('should not remove non-auth keys', () => {
      localStorage.setItem('other_key', 'value');
      localStorage.setItem(`${AUTH_KEY_PREFIX}refresh_token`, 'rt');

      clearTokens();

      expect(localStorage.getItem('other_key')).toBe('value');
    });
  });

  describe('getTokenExpiry', () => {
    it('should return null when no expiry is stored', () => {
      expect(getTokenExpiry()).toBeNull();
    });

    it('should return the stored expiry as a number', () => {
      localStorage.setItem(`${AUTH_KEY_PREFIX}token_expiry`, '1234567890');
      expect(getTokenExpiry()).toBe(1234567890);
    });
  });
});
