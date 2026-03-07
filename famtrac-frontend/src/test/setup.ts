import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock environment variables for tests
vi.stubEnv('VITE_API_BASE_URL', 'http://localhost:8080');
vi.stubEnv('VITE_COGNITO_DOMAIN', 'test.auth.us-east-1.amazoncognito.com');
vi.stubEnv('VITE_COGNITO_CLIENT_ID', 'test-client-id');
vi.stubEnv('VITE_COGNITO_REDIRECT_URI', 'http://localhost:5173/callback');
vi.stubEnv('VITE_COGNITO_LOGOUT_URI', 'http://localhost:5173');
vi.stubEnv('VITE_COGNITO_SCOPE', 'openid email profile');
