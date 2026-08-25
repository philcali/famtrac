import React, { useEffect, useState, useCallback, useRef } from 'react';
import {
  getRefreshToken,
  refreshAccessToken,
  storeRefreshToken,
  clearTokens,
  getTokenExpiry,
} from './tokenService';
import { buildLoginUrl, buildLogoutUrl } from '../config/cognito';
import { AuthContext } from './AuthContext';
import type { AuthContextValue, CognitoUser } from './types';

const REFRESH_RETRY_DELAY_MS = 30_000;

interface AuthProviderProps {
  children: React.ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [user, setUser] = useState<CognitoUser | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const refreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const parseUserFromIdToken = useCallback((token: string): CognitoUser | null => {
    try {
      const parts = token.split('.');
      if (parts.length !== 3) return null;
      const payload = JSON.parse(atob(parts[1]));
      if (payload['cognito:username']) {
        payload['username'] = payload['cognito:username'];
      }
      return payload as CognitoUser;
    } catch {
      console.error('Failed to parse ID token');
      return null;
    }
  }, []);

  const cancelRefreshTimer = useCallback(() => {
    if (refreshTimerRef.current !== null) {
      clearTimeout(refreshTimerRef.current);
      refreshTimerRef.current = null;
    }
  }, []);

  const handleSessionExpired = useCallback(() => {
    cancelRefreshTimer();
    clearTokens();
    setAccessToken(null);
    setIsAuthenticated(false);
    setUser(null);
  }, [cancelRefreshTimer]);

  const scheduleRefreshRef = useRef<(expiresIn: number) => void>(() => {});

  const scheduleRefresh = useCallback(
    (expiresIn: number) => {
      cancelRefreshTimer();
      const delayMs = expiresIn > 300 ? (expiresIn - 300) * 1000 : 0;

      refreshTimerRef.current = setTimeout(async () => {
        const storedRefresh = getRefreshToken();
        if (!storedRefresh) {
          handleSessionExpired();
          window.location.href = await buildLoginUrl();
          return;
        }

        let tokens = await refreshAccessToken(storedRefresh);

        // Retry once after 30s on failure
        if (!tokens) {
          await new Promise((r) => setTimeout(r, REFRESH_RETRY_DELAY_MS));
          tokens = await refreshAccessToken(storedRefresh);
        }

        if (tokens) {
          setAccessToken(tokens.access_token);
          if (tokens.refresh_token) {
            storeRefreshToken(tokens.refresh_token, tokens.expires_in);
          }
          const userData = parseUserFromIdToken(tokens.id_token);
          setUser(userData);
          scheduleRefreshRef.current(tokens.expires_in);
        } else {
          handleSessionExpired();
          window.location.href = await buildLoginUrl();
        }
      }, delayMs);
    },
    [cancelRefreshTimer, handleSessionExpired, parseUserFromIdToken]
  );

  // Keep the ref in sync with the latest scheduleRefresh
  useEffect(() => {
    scheduleRefreshRef.current = scheduleRefresh;
  }, [scheduleRefresh]);

  // Initialize authentication state
  useEffect(() => {
    const initAuth = async () => {
      try {
        const storedRefresh = getRefreshToken();
        if (!storedRefresh) {
          // No valid refresh token (missing or session expired beyond 30 days)
          return;
        }

        const tokens = await refreshAccessToken(storedRefresh);
        if (tokens) {
          setAccessToken(tokens.access_token);
          if (tokens.refresh_token) {
            storeRefreshToken(tokens.refresh_token, tokens.expires_in);
          }
          setIsAuthenticated(true);
          const userData = parseUserFromIdToken(tokens.id_token);
          setUser(userData);
          scheduleRefresh(tokens.expires_in);
        } else {
          clearTokens();
        }
      } catch (error) {
        console.error('Auth initialization error:', error);
        clearTokens();
      } finally {
        setIsLoading(false);
      }
    };

    initAuth();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Cleanup refresh timer on unmount
  useEffect(() => {
    return () => cancelRefreshTimer();
  }, [cancelRefreshTimer]);

  // Listen for auth:expired events
  useEffect(() => {
    const onExpired = async () => {
      handleSessionExpired();
      window.location.href = await buildLoginUrl();
    };

    window.addEventListener('auth:expired', onExpired);
    return () => window.removeEventListener('auth:expired', onExpired);
  }, [handleSessionExpired]);

  const login = useCallback(async () => {
    sessionStorage.setItem('auth_redirect_url', window.location.pathname);
    window.location.href = await buildLoginUrl();
  }, []);

  const logout = useCallback(() => {
    cancelRefreshTimer();
    clearTokens();
    setAccessToken(null);
    setIsAuthenticated(false);
    setUser(null);
    window.location.href = buildLogoutUrl();
  }, [cancelRefreshTimer]);

  const getToken = useCallback(async (): Promise<string | null> => {
    if (accessToken) {
      const expiry = getTokenExpiry();
      if (expiry && Date.now() < expiry) {
        return accessToken;
      }
    }

    // Token missing or expired — try refresh
    const storedRefresh = getRefreshToken();
    if (!storedRefresh) {
      handleSessionExpired();
      return null;
    }

    const tokens = await refreshAccessToken(storedRefresh);
    if (tokens) {
      setAccessToken(tokens.access_token);
      if (tokens.refresh_token) {
        storeRefreshToken(tokens.refresh_token, tokens.expires_in);
      }
      scheduleRefresh(tokens.expires_in);
      return tokens.access_token;
    }

    handleSessionExpired();
    return null;
  }, [accessToken, handleSessionExpired, scheduleRefresh]);

  const value: AuthContextValue = {
    isAuthenticated,
    isLoading,
    user,
    login,
    logout,
    getToken,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
