export interface CognitoUser {
  sub: string;
  username?: string;
  email?: string;
  email_verified?: boolean;
  [key: string]: unknown;
}

export interface AuthContextValue {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: CognitoUser | null;
  login: () => Promise<void>;
  logout: () => void;
  getToken: () => Promise<string | null>;
}
