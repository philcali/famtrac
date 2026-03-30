# Implementation Plan: Cognito Session Caching

## Overview

Migrate the famtrac-frontend authentication from OAuth implicit flow with sessionStorage to Authorization Code flow with PKCE using localStorage-backed persistent sessions. Implementation proceeds bottom-up: CDK infra first, then token service, cognito config, auth provider, callback page, and finally cleanup of old code and tests.

## Tasks

- [x] 1. Update CDK infrastructure for public client PKCE flow
  - [x] 1.1 Modify `famtrac-infra/lib/auth/FamtracAuthorization.ts` UserPoolClient configuration
    - Change `generateSecret: true` to `generateSecret: false`
    - Remove `implicitCodeGrant: true` from `oAuth.flows`
    - Keep `authorizationCodeGrant: true`
    - Keep all token validity settings unchanged (`accessTokenValidity: 1 day`, `idTokenValidity: 1 day`, `refreshTokenValidity: 365 days`)
    - Keep existing `callbackUrls` using `/login` path unchanged
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [-] 2. Rewrite TokenService for split storage and PKCE
  - [x] 2.1 Rewrite `famtrac-frontend/src/auth/tokenService.ts` with new storage strategy
    - Define constants: `AUTH_KEY_PREFIX = 'famtrac_auth_'`, key names for refresh token, token expiry, session timestamp, code verifier
    - Define `SESSION_LIFETIME_MS = 30 * 24 * 60 * 60 * 1000`
    - Implement `generateCodeVerifier()` using `crypto.getRandomValues()` to produce a 128-char random string
    - Implement `deriveCodeChallenge(verifier)` using `SubtleCrypto.digest('SHA-256')` with base64url encoding
    - Implement `storeCodeVerifier(verifier)` and `getAndClearCodeVerifier()` using localStorage
    - Implement `exchangeCodeForTokens(code)` that POSTs to Cognito `/oauth2/token` with `grant_type=authorization_code`, code, code_verifier, redirect_uri, client_id
    - Implement `storeRefreshToken(refreshToken, expiresIn)` that writes refresh token, expiry timestamp, and session timestamp (only if not already set) to localStorage
    - Implement `getRefreshToken()` that checks 30-day session lifetime before returning the token, clears all auth keys if expired
    - Implement `getTokenExpiry()` and `isSessionExpired()` helpers
    - Rewrite `refreshAccessToken()` to use localStorage-based refresh token
    - Rewrite `clearTokens()` to remove all keys with `famtrac_auth_` prefix from localStorage
    - Remove `parseTokensFromUrl()`, `storeTokens()`, `getAccessToken()`, `getIdToken()`, and all sessionStorage usage
    - _Requirements: 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 5.3, 5.4, 7.1, 7.3, 7.4_

  - [ ]* 2.2 Write property test: PKCE code challenge derivation is deterministic
    - **Property 2: PKCE code challenge derivation is deterministic**
    - **Validates: Requirement 1.2**

  - [ ]* 2.3 Write property test: Token storage round-trip
    - **Property 4: Token storage round-trip**
    - **Validates: Requirements 2.1, 2.2, 2.6, 5.3**

  - [ ]* 2.4 Write property test: Session lifetime enforcement
    - **Property 5: Session lifetime enforcement**
    - **Validates: Requirements 2.3, 2.4, 5.1**

  - [ ]* 2.5 Write property test: Refresh preserves session timestamp
    - **Property 6: Refresh preserves session timestamp**
    - **Validates: Requirement 2.5**

  - [ ]* 2.6 Write property test: Logout clears all authentication keys
    - **Property 8: Logout clears all authentication keys**
    - **Validates: Requirements 6.1, 7.3**

  - [ ]* 2.7 Write property test: localStorage never contains access or ID tokens
    - **Property 9: localStorage never contains access or ID tokens**
    - **Validates: Requirement 7.1**

  - [ ]* 2.8 Write property test: All authentication keys use consistent prefix
    - **Property 10: All authentication keys use consistent prefix**
    - **Validates: Requirement 7.4**

- [x] 3. Update CognitoConfig for PKCE authorization code flow
  - [x] 3.1 Rewrite `famtrac-frontend/src/config/cognito.ts` `buildLoginUrl` to use authorization code flow with PKCE
    - Change `response_type` from `'token'` to `'code'`
    - Generate code verifier via `generateCodeVerifier()`, store it via `storeCodeVerifier()`
    - Derive code challenge via `deriveCodeChallenge()` and add `code_challenge` and `code_challenge_method=S256` to URL params
    - Make `buildLoginUrl()` async (returns `Promise<string>`) since `deriveCodeChallenge` uses SubtleCrypto
    - Keep `buildLogoutUrl()` unchanged
    - _Requirements: 1.1, 1.2, 1.6_

  - [ ]* 3.2 Write property test: Login URL uses authorization code flow with PKCE
    - **Property 1: Login URL uses authorization code flow with PKCE**
    - **Validates: Requirements 1.1, 1.6**

  - [ ]* 3.3 Write property test: Token exchange request includes required PKCE parameters
    - **Property 3: Token exchange request includes required PKCE parameters**
    - **Validates: Requirement 1.3**

- [ ] 4. Checkpoint - Verify token service and config
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Rewrite AuthProvider for in-memory tokens and proactive refresh
  - [x] 5.1 Update `famtrac-frontend/src/auth/types.ts` to make `login()` async
    - Change `login: () => void` to `login: () => Promise<void>` in `AuthContextValue` interface
    - _Requirements: 1.1_

  - [x] 5.2 Rewrite `famtrac-frontend/src/auth/AuthProvider.tsx` with new auth lifecycle
    - Add in-memory state for `accessToken`, `idToken` (not persisted to storage)
    - Change initialization: check for stored refresh token via `getRefreshToken()`, verify 30-day session, use `refreshAccessToken()` to get fresh access/ID tokens, store in React state
    - Implement `scheduleRefresh(expiresIn)`: set timer for `(expiresIn - 300) * 1000` ms (or immediately if ≤ 300s)
    - On refresh success: update in-memory tokens, call `storeRefreshToken()` with new metadata, reschedule
    - On refresh failure: retry once after 30-second delay; on retry failure call `clearTokens()` and redirect to login
    - Make `login()` async, call async `buildLoginUrl()`, store redirect URL in sessionStorage before redirecting
    - Update `logout()` to call `clearTokens()` and redirect to Cognito logout endpoint via `buildLogoutUrl()`
    - Update `getToken()` to return in-memory `accessToken` and handle refresh if expired
    - Cancel refresh timer on component unmount via useEffect cleanup
    - Listen for `auth:expired` event to clear session and redirect to login
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 6.1, 6.2, 6.3, 7.1, 7.2_

  - [ ]* 5.3 Write property test: Proactive refresh scheduling
    - **Property 7: Proactive refresh scheduling**
    - **Validates: Requirement 4.1**

- [x] 6. Rewrite CallbackPage for authorization code exchange
  - [x] 6.1 Rewrite `famtrac-frontend/src/pages/CallbackPage.tsx` to handle authorization code
    - Read `code` from URL query parameters (`window.location.search`) instead of hash fragment
    - Call `exchangeCodeForTokens(code)` from tokenService
    - On success: store refresh token via `storeRefreshToken()`, dispatch a custom event or use a shared mechanism to pass access/ID tokens to AuthProvider, redirect to preserved URL or `/`
    - On failure: call `clearTokens()` to remove any partial state, redirect to `/`
    - Handle missing `code` parameter by redirecting to `/`
    - Remove all `parseTokensFromUrl()` usage
    - _Requirements: 1.3, 1.4, 1.5_

- [x] 7. Update existing tests for new auth flow
  - [x] 7.1 Rewrite `famtrac-frontend/src/auth/tokenService.test.ts` for new TokenService API
    - Replace all sessionStorage-based tests with localStorage-based tests
    - Test `storeRefreshToken` / `getRefreshToken` round-trip
    - Test `clearTokens` removes all `famtrac_auth_` prefixed keys
    - Test `isSessionExpired` returns true for sessions older than 30 days
    - Test `generateCodeVerifier` produces valid length and character set
    - Test `getAndClearCodeVerifier` returns verifier and removes it from storage
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 7.3, 7.4_

  - [x] 7.2 Rewrite `famtrac-frontend/src/auth/AuthProvider.test.tsx` for new AuthProvider behavior
    - Test initialization with valid refresh token sets authenticated state (mock `refreshAccessToken`)
    - Test initialization with expired session (>30 days) redirects to login
    - Test logout clears localStorage and redirects to Cognito logout URL
    - Test `auth:expired` event triggers cleanup and redirect
    - Test unmount cancels refresh timer
    - Remove tests that reference `parseTokensFromUrl` or hash-based token extraction
    - _Requirements: 4.1, 4.5, 5.1, 5.2, 6.1, 6.2, 6.3_

- [ ] 8. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- The callback route is `/login` (not `/callback`) — this matches the CDK infra and App.tsx routing
- Access tokens and ID tokens are held in memory only (AuthProvider state), never written to localStorage
- The `.env.example` has `VITE_COGNITO_REDIRECT_URI` pointing to `/callback` which is inconsistent with the actual `/login` route, but this is a pre-existing issue outside the scope of this feature
- Property tests use fast-check (already in devDependencies) and run via Vitest
