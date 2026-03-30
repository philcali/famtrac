# Requirements Document

## Introduction

The famtrac-frontend currently uses sessionStorage for Cognito token persistence and the OAuth implicit flow for authentication. This combination causes users to lose their session every time they close a browser tab, requiring re-login. The goal is to implement a persistent session caching strategy that keeps users logged in for up to 30 days by switching to the Authorization Code flow with PKCE (to reliably obtain refresh tokens) and using localStorage for persistent token storage with appropriate security controls.

## Glossary

- **Token_Service**: The frontend module (`tokenService.ts`) responsible for storing, retrieving, refreshing, and clearing OAuth tokens.
- **Auth_Provider**: The React context provider (`AuthProvider.tsx`) that manages authentication state, token lifecycle, and exposes auth operations to the application.
- **Cognito_Config**: The frontend module (`cognito.ts`) that builds Cognito OAuth URLs and provides configuration.
- **Callback_Handler**: The page component (`CallbackPage.tsx`) that processes OAuth redirects and extracts tokens.
- **Protected_Route**: The route guard component (`ProtectedRoute.tsx`) that redirects unauthenticated users to login.
- **API_Client**: The HTTP client (`client.ts`) that attaches auth tokens to API requests and handles 401 responses.
- **CDK_Auth_Stack**: The CDK construct (`FamtracAuthorization.ts`) that defines the Cognito UserPool and UserPoolClient configuration.
- **Access_Token**: A short-lived JWT issued by Cognito used to authorize API requests (currently valid for 24 hours).
- **Refresh_Token**: A long-lived token issued by Cognito used to obtain new access and ID tokens without re-authentication (currently valid for 365 days in infrastructure).
- **PKCE**: Proof Key for Code Exchange, a security extension to the Authorization Code flow that eliminates the need for a client secret in public clients.
- **Session_Timestamp**: A stored timestamp recording when the user's persistent session began, used to enforce the 30-day session lifetime cap.

## Requirements

### Requirement 1: Switch to Authorization Code Flow with PKCE

**User Story:** As a developer, I want the frontend to use the Authorization Code flow with PKCE instead of the implicit flow, so that the application reliably receives refresh tokens from Cognito.

#### Acceptance Criteria

1. WHEN a user initiates login, THE Cognito_Config SHALL build an authorization URL using `response_type=code` and include a PKCE `code_challenge` parameter.
2. WHEN the Cognito_Config builds a login URL, THE Cognito_Config SHALL generate a cryptographically random `code_verifier`, derive a `code_challenge` using SHA-256, and store the `code_verifier` for use during the token exchange.
3. WHEN the Callback_Handler receives an authorization code from Cognito, THE Callback_Handler SHALL exchange the authorization code for tokens by calling the Cognito `/oauth2/token` endpoint with the `code_verifier`.
4. WHEN the token exchange succeeds, THE Callback_Handler SHALL receive an access token, an ID token, and a refresh token from Cognito.
5. WHEN the token exchange fails, THE Callback_Handler SHALL clear any partial authentication state and redirect the user to the home page.
6. THE Cognito_Config SHALL cease using `response_type=token` (implicit flow) for login URL construction.

### Requirement 2: Persistent Token Storage

**User Story:** As a user, I want my authentication tokens to persist across browser tab closures and browser restarts, so that I do not have to log in again every time I open the application.

#### Acceptance Criteria

1. WHEN the Token_Service stores tokens, THE Token_Service SHALL write tokens to localStorage instead of sessionStorage.
2. WHEN the Token_Service stores tokens, THE Token_Service SHALL also store a Session_Timestamp recording the time the session was originally established.
3. WHEN the Token_Service reads tokens from localStorage, THE Token_Service SHALL verify that the stored Session_Timestamp is within the 30-day session lifetime before returning the tokens.
4. IF the Session_Timestamp indicates the session is older than 30 days, THEN THE Token_Service SHALL clear all stored tokens and session data from localStorage.
5. WHEN a token refresh succeeds, THE Token_Service SHALL update the stored access token and ID token while preserving the original Session_Timestamp.
6. THE Token_Service SHALL store the token expiry timestamp alongside the tokens in localStorage.

### Requirement 3: CDK Infrastructure Update

**User Story:** As a developer, I want the Cognito UserPoolClient to be configured for public client PKCE flow without a client secret, so that the frontend can perform the authorization code exchange securely.

#### Acceptance Criteria

1. THE CDK_Auth_Stack SHALL configure the UserPoolClient with `generateSecret: false` to support public client PKCE flow.
2. THE CDK_Auth_Stack SHALL retain `authorizationCodeGrant: true` in the OAuth flow configuration.
3. THE CDK_Auth_Stack SHALL remove `implicitCodeGrant: true` from the OAuth flow configuration.
4. THE CDK_Auth_Stack SHALL retain the existing `refreshTokenValidity` of 365 days.
5. THE CDK_Auth_Stack SHALL retain the existing `accessTokenValidity` of 1 day and `idTokenValidity` of 1 day.

### Requirement 4: Proactive Token Refresh

**User Story:** As a user, I want my access token to be refreshed automatically before it expires, so that I experience uninterrupted API access without being logged out unexpectedly.

#### Acceptance Criteria

1. WHEN the Auth_Provider initializes, THE Auth_Provider SHALL schedule a proactive token refresh to occur 5 minutes before the current access token expires.
2. WHEN a scheduled token refresh succeeds, THE Auth_Provider SHALL store the new tokens and schedule the next proactive refresh based on the new token expiry.
3. IF a scheduled token refresh fails, THEN THE Auth_Provider SHALL retry the refresh once after a 30-second delay.
4. IF the retry also fails, THEN THE Auth_Provider SHALL clear the session and redirect the user to the Cognito login page.
5. WHEN the Auth_Provider component unmounts, THE Auth_Provider SHALL cancel any pending refresh timers.

### Requirement 5: Session Lifetime Enforcement

**User Story:** As a product owner, I want user sessions to be capped at 30 days regardless of activity, so that security is maintained through periodic re-authentication.

#### Acceptance Criteria

1. WHEN the Auth_Provider initializes and finds stored tokens, THE Auth_Provider SHALL check the Session_Timestamp to determine whether the 30-day session lifetime has been exceeded.
2. IF the session lifetime has been exceeded, THEN THE Auth_Provider SHALL clear all stored tokens and redirect the user to the Cognito login page.
3. WHEN a new session is established after login, THE Token_Service SHALL record a new Session_Timestamp set to the current time.
4. THE Token_Service SHALL treat the 30-day session lifetime as a configurable constant defined in a single location.

### Requirement 6: Logout and Session Cleanup

**User Story:** As a user, I want logging out to completely clear my persistent session data, so that my account is secure on shared devices.

#### Acceptance Criteria

1. WHEN the user triggers logout, THE Auth_Provider SHALL clear all tokens, Session_Timestamp, and PKCE-related data from localStorage.
2. WHEN the user triggers logout, THE Auth_Provider SHALL redirect the user to the Cognito logout endpoint to invalidate the server-side session.
3. WHEN the API_Client receives a 401 response, THE API_Client SHALL dispatch an `auth:expired` event, and THE Auth_Provider SHALL clear all stored session data and redirect to login.

### Requirement 7: Security Controls for Persistent Storage

**User Story:** As a developer, I want appropriate security measures around persistent token storage, so that the risk of token theft from localStorage is mitigated.

#### Acceptance Criteria

1. THE Token_Service SHALL store only the refresh token and token metadata (expiry, session timestamp) in localStorage; access tokens and ID tokens SHALL be held in memory within the Auth_Provider when possible.
2. WHEN the Auth_Provider initializes with a stored refresh token, THE Auth_Provider SHALL immediately use the refresh token to obtain fresh access and ID tokens rather than reading stale access tokens from storage.
3. WHEN the Token_Service clears session data, THE Token_Service SHALL remove all authentication-related keys from localStorage.
4. THE Token_Service SHALL use a consistent key prefix for all authentication-related localStorage entries to enable complete cleanup.
