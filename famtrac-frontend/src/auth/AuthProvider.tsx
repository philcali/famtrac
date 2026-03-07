import React, { createContext, useEffect, useState, useCallback } from 'react';
import {
  parseTokensFromUrl,
  storeTokens,
  getAccessToken,
  clearTokens,
  isTokenExpired,
  getRefreshToken,
  refreshAccessToken,
  getIdToken,
} from './tokenService';
import { buildLoginUrl } from '../config/cognito';

interface CognitoUser {
  sub: string;
  email?: string;
  email_verified?: boolean;
  [key: string]: unknown;
}

export interface AuthContextValue {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: CognitoUser | null;
  login: () => void;
  logout: () => void;
  getToken: () => Promise<string | null>;
}

export const AuthContext = createContext<AuthContextValue | undefined>(
  undefined
);

interface AuthProviderProps {
  children: React.ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [user, setUser] = useState<CognitoUser | null>(null);

  // Parse user info from ID token
  const parseUserFromIdToken = useCallback((idToken: string): CognitoUser | null => {
    try {
      // JWT tokens have 3 parts separated by dots
      const parts = idToken.split('.');
      if (parts.length !== 3) {
        return null;
      }

      // Decode the payload (second part)
      const payload = JSON.parse(atob(parts[1]));
      return payload as CognitoUser;
    } catch (error) {
      console.error('Failed to parse ID token:', error);
      return null;
    }
  }, []);

  // Initialize authentication state
  useEffect(() => {
    const initAuth = async () => {
      try {
        // Check if we're returning from OAuth callback
        const tokensFromUrl = parseTokensFromUrl();
        if (tokensFromUrl) {
          storeTokens(tokensFromUrl);
          // Clear the hash from URL
          window.history.replaceState(
            null,
            '',
            window.location.pathname + window.location.search
          );
        }

        // Check for existing valid token
        const token = getAccessToken();
        if (token && !isTokenExpired()) {
          setIsAuthenticated(true);
          const idToken = getIdToken();
          if (idToken) {
            const userData = parseUserFromIdToken(idToken);
            setUser(userData);
          }
        } else if (token && isTokenExpired()) {
          // Try to refresh the token
          const refreshToken = getRefreshToken();
          if (refreshToken) {
            const newTokens = await refreshAccessToken(refreshToken);
            if (newTokens) {
              storeTokens(newTokens);
              setIsAuthenticated(true);
              const userData = parseUserFromIdToken(newTokens.id_token);
              setUser(userData);
            } else {
              clearTokens();
              setIsAuthenticated(false);
              setUser(null);
            }
          } else {
            clearTokens();
            setIsAuthenticated(false);
            setUser(null);
          }
        }
      } catch (error) {
        console.error('Auth initialization error:', error);
        clearTokens();
        setIsAuthenticated(false);
        setUser(null);
      } finally {
        setIsLoading(false);
      }
    };

    initAuth();
  }, [parseUserFromIdToken]);

  // Listen for auth expiration events
  useEffect(() => {
    const handleAuthExpired = () => {
      clearTokens();
      setIsAuthenticated(false);
      setUser(null);
      // Redirect to login
      login();
    };

    window.addEventListener('auth:expired', handleAuthExpired);
    return () => {
      window.removeEventListener('auth:expired', handleAuthExpired);
    };
  }, []);

  const login = useCallback(() => {
    // Save current URL for post-login redirect
    sessionStorage.setItem('auth_redirect_url', window.location.pathname);
    // Redirect to Cognito hosted UI
    window.location.href = buildLoginUrl();
  }, []);

  const logout = useCallback(() => {
    clearTokens();
    setIsAuthenticated(false);
    setUser(null);
    // For now, just redirect to home. In production, you'd redirect to Cognito logout
    window.location.href = '/';
  }, []);

  const getToken = useCallback(async (): Promise<string | null> => {
    const token = getAccessToken();
    
    if (!token) {
      return null;
    }

    // Check if token is expired
    if (isTokenExpired()) {
      const refreshToken = getRefreshToken();
      if (refreshToken) {
        const newTokens = await refreshAccessToken(refreshToken);
        if (newTokens) {
          storeTokens(newTokens);
          return newTokens.access_token;
        }
      }
      // Token expired and couldn't refresh
      clearTokens();
      setIsAuthenticated(false);
      setUser(null);
      return null;
    }

    return token;
  }, []);

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
