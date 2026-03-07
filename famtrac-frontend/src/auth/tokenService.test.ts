import { describe, it, expect, beforeEach } from 'vitest';
import {
  parseTokensFromUrl,
  storeTokens,
  getAccessToken,
  clearTokens,
  isTokenExpired,
  type CognitoTokens,
} from './tokenService';

describe('tokenService', () => {
  beforeEach(() => {
    // Clear sessionStorage before each test
    sessionStorage.clear();
    // Reset window.location.hash
    window.location.hash = '';
  });

  describe('parseTokensFromUrl', () => {
    it('should parse tokens from URL hash', () => {
      window.location.hash =
        '#access_token=test_access&id_token=test_id&expires_in=3600&token_type=Bearer';

      const tokens = parseTokensFromUrl();

      expect(tokens).toEqual({
        access_token: 'test_access',
        id_token: 'test_id',
        expires_in: 3600,
        token_type: 'Bearer',
      });
    });

    it('should return null when no hash present', () => {
      const tokens = parseTokensFromUrl();
      expect(tokens).toBeNull();
    });

    it('should return null when required tokens are missing', () => {
      window.location.hash = '#access_token=test_access';
      const tokens = parseTokensFromUrl();
      expect(tokens).toBeNull();
    });
  });

  describe('storeTokens and getAccessToken', () => {
    it('should store and retrieve tokens', () => {
      const tokens: CognitoTokens = {
        access_token: 'test_access',
        id_token: 'test_id',
        expires_in: 3600,
      };

      storeTokens(tokens);
      const accessToken = getAccessToken();

      expect(accessToken).toBe('test_access');
    });

    it('should return null when no tokens stored', () => {
      const accessToken = getAccessToken();
      expect(accessToken).toBeNull();
    });
  });

  describe('clearTokens', () => {
    it('should clear all tokens from storage', () => {
      const tokens: CognitoTokens = {
        access_token: 'test_access',
        id_token: 'test_id',
        expires_in: 3600,
      };

      storeTokens(tokens);
      clearTokens();
      const accessToken = getAccessToken();

      expect(accessToken).toBeNull();
    });
  });

  describe('isTokenExpired', () => {
    it('should return true when no token stored', () => {
      expect(isTokenExpired()).toBe(true);
    });

    it('should return false for non-expired token', () => {
      const tokens: CognitoTokens = {
        access_token: 'test_access',
        id_token: 'test_id',
        expires_in: 3600, // 1 hour
      };

      storeTokens(tokens);
      expect(isTokenExpired()).toBe(false);
    });

    it('should return true for expired token', () => {
      const tokens: CognitoTokens = {
        access_token: 'test_access',
        id_token: 'test_id',
        expires_in: 0, // Already expired
      };

      storeTokens(tokens);
      // Wait a tiny bit to ensure expiry
      expect(isTokenExpired()).toBe(true);
    });
  });
});
