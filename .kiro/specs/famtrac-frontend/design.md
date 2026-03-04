# Design Document: famtrac-frontend

## Overview

The famtrac-frontend is a modern, lightweight single-page application (SPA) built to provide a user interface for managing families, dependents, and their activities. The application prioritizes simplicity and uses contemporary web development tools while integrating with the existing famtrac-backend REST API and AWS Cognito for authentication.

### Technology Stack

**Build Tool & Framework:**
- **Vite** - Modern, fast build tool with excellent developer experience
- **React 18** - UI library with hooks for state management
- **TypeScript** - Type safety and better developer experience

**Routing:**
- **React Router v6** - Declarative routing with modern hooks API

**HTTP Client:**
- **Fetch API** - Native browser API, no additional dependencies needed

**Authentication:**
- **Custom Cognito integration** - Lightweight implementation using Cognito hosted UI and native fetch for token handling

**Form Management:**
- **Native React state** - Simple forms don't require heavy libraries
- **Custom validation hooks** - Lightweight, reusable validation logic

**Styling:**
- **react-bootstrap** - Pre-built, accessible React components with Bootstrap styling
- **Bootstrap 5** - Responsive grid system and utility classes
- **Custom CSS** - Optional custom styles to augment Bootstrap's look and feel

**Development Tools:**
- **ESLint** - Code quality
- **Prettier** - Code formatting
- **Vitest** - Fast unit testing (Vite-native)

### Design Principles

1. **Simplicity First** - Avoid over-engineering; use native browser APIs where possible
2. **Minimal Dependencies** - Only add libraries that provide significant value
3. **Type Safety** - Leverage TypeScript for catching errors early
4. **Responsive by Default** - Mobile-first CSS approach
5. **Progressive Enhancement** - Core functionality works, enhancements improve experience

## Architecture

### Application Structure

```
famtrac-frontend/
├── src/
│   ├── main.tsx                 # Application entry point
│   ├── App.tsx                  # Root component with routing
│   ├── config/
│   │   ├── environment.ts       # Environment configuration
│   │   └── cognito.ts           # Cognito OAuth configuration
│   ├── api/
│   │   ├── client.ts            # HTTP client wrapper
│   │   ├── families.ts          # Family API methods
│   │   ├── dependents.ts        # Dependent API methods
│   │   ├── activities.ts        # Activity API methods
│   │   └── types.ts             # API request/response types
│   ├── auth/
│   │   ├── AuthProvider.tsx     # Authentication context provider
│   │   ├── useAuth.ts           # Authentication hook
│   │   ├── tokenService.ts      # Token parsing, storage, and refresh
│   │   └── ProtectedRoute.tsx   # Route guard component
│   ├── components/
│   │   ├── common/              # Reusable UI components
│   │   │   ├── Button.tsx
│   │   │   ├── Input.tsx
│   │   │   ├── LoadingSpinner.tsx
│   │   │   ├── ErrorMessage.tsx
│   │   │   ├── SuccessMessage.tsx
│   │   │   └── ConfirmDialog.tsx
│   │   ├── families/            # Family-specific components
│   │   │   ├── FamilyList.tsx
│   │   │   ├── FamilyForm.tsx
│   │   │   └── FamilyCard.tsx
│   │   ├── dependents/          # Dependent-specific components
│   │   │   ├── DependentList.tsx
│   │   │   ├── DependentForm.tsx
│   │   │   └── DependentCard.tsx
│   │   └── activities/          # Activity-specific components
│   │       ├── ActivityList.tsx
│   │       ├── ActivityForm.tsx
│   │       ├── ActivityCard.tsx
│   │       └── ActivityFilters.tsx
│   ├── pages/
│   │   ├── FamiliesPage.tsx     # List all families
│   │   ├── FamilyDetailPage.tsx # View single family
│   │   ├── DependentDetailPage.tsx # View single dependent
│   │   ├── NotFoundPage.tsx     # 404 page
│   │   └── ErrorPage.tsx        # Error boundary page
│   ├── hooks/
│   │   ├── useApi.ts            # Generic API call hook
│   │   ├── useForm.ts           # Form state management
│   │   └── useValidation.ts     # Form validation logic
│   ├── utils/
│   │   ├── validation.ts        # Validation functions
│   │   ├── dateUtils.ts         # Date formatting and calculations
│   │   └── errorHandling.ts     # Error parsing utilities
│   ├── styles/
│   │   └── custom.css           # Optional custom styles to augment Bootstrap
│   └── types/
│       └── domain.ts            # Domain model types
├── public/
│   └── index.html
├── .env.development
├── .env.production
├── vite.config.ts
├── tsconfig.json
└── package.json
```

### Component Architecture

The application follows a layered architecture:

1. **Pages Layer** - Route-level components that compose features
2. **Components Layer** - Reusable UI components organized by domain
3. **Hooks Layer** - Custom React hooks for shared logic
4. **API Layer** - Backend communication abstraction
5. **Auth Layer** - Authentication state and guards
6. **Utils Layer** - Pure functions for validation, formatting, etc.

### State Management Strategy

Given the application's simplicity, we avoid heavy state management libraries:

- **React Context** - For global state (authentication)
- **Component State** - For local UI state (forms, modals)
- **URL State** - For navigation and filters (via React Router)
- **Server State** - Fetched on-demand, not cached (simple refetch pattern)

This approach keeps the application simple while meeting all requirements. If the application grows significantly, we can introduce React Query or similar libraries later.

### Bootstrap Styling Strategy

The application uses react-bootstrap for UI components with optional customization:

**Base Setup:**
```typescript
// main.tsx
import 'bootstrap/dist/css/bootstrap.min.css';
import './styles/custom.css'; // Optional overrides
```

**Customization Approach:**
- Use Bootstrap's default theme initially for rapid development
- Add custom CSS in `styles/custom.css` for brand-specific styling
- Override Bootstrap variables if deeper customization is needed later
- Leverage Bootstrap utility classes for spacing, colors, and responsive design

**Example Custom Styles:**
```css
/* styles/custom.css */
:root {
  --primary-color: #your-brand-color;
}

/* Override Bootstrap primary color if needed */
.btn-primary {
  background-color: var(--primary-color);
  border-color: var(--primary-color);
}

/* Custom spacing or layout adjustments */
.family-card {
  margin-bottom: 1rem;
}
```

**Benefits:**
- Start with Bootstrap defaults for speed
- Easy to customize incrementally
- Consistent component behavior and accessibility
- Responsive grid system handles mobile/desktop layouts

## Components and Interfaces

### API Client

The API client provides a thin wrapper around the Fetch API with authentication and error handling.

```typescript
// api/client.ts
interface ApiClientConfig {
  baseURL: string;
  timeout: number;
}

interface ApiResponse<T> {
  data?: T;
  error?: string;
}

class ApiClient {
  private config: ApiClientConfig;
  private getAuthToken: () => Promise<string | null>;

  async get<T>(path: string): Promise<ApiResponse<T>>;
  async post<T>(path: string, body: unknown): Promise<ApiResponse<T>>;
  async put<T>(path: string, body: unknown): Promise<ApiResponse<T>>;
  async delete<T>(path: string): Promise<ApiResponse<T>>;
}
```

**Key Responsibilities:**
- Construct full URLs from base URL and path
- Inject Authorization header with JWT token
- Set Content-Type headers appropriately
- Parse JSON responses
- Handle HTTP errors and timeouts
- Extract error messages from error responses

### Authentication Provider

The authentication system uses React Context to provide auth state throughout the app.

```typescript
// auth/AuthProvider.tsx
interface AuthContextValue {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: CognitoUser | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  getToken: () => Promise<string | null>;
}

// auth/useAuth.ts
function useAuth(): AuthContextValue;

// auth/ProtectedRoute.tsx
interface ProtectedRouteProps {
  children: React.ReactNode;
}
```

**Authentication Flow:**
1. App loads, AuthProvider checks for tokens in sessionStorage
2. If no tokens, redirect to Cognito hosted UI with OAuth parameters
3. After successful login, Cognito redirects back with tokens in URL hash/query
4. Parse tokens from callback URL, store in sessionStorage
5. Provide access token to API client for all requests
6. On token expiration (401 response), attempt refresh with refresh token
7. If refresh fails, clear tokens and redirect to Cognito hosted UI

**Token Handling:**
```typescript
// auth/tokenService.ts
interface CognitoTokens {
  access_token: string;
  id_token: string;
  refresh_token?: string;
  expires_in: number;
}

function parseTokensFromUrl(): CognitoTokens | null;
function storeTokens(tokens: CognitoTokens): void;
function getAccessToken(): string | null;
function clearTokens(): void;
function isTokenExpired(token: string): boolean;
function refreshAccessToken(refreshToken: string): Promise<CognitoTokens | null>;
```

**Cognito Configuration:**
```typescript
// config/cognito.ts
interface CognitoConfig {
  domain: string;        // e.g., "famtrac.auth.us-east-1.amazoncognito.com"
  clientId: string;      // Cognito app client ID
  redirectUri: string;   // e.g., "http://localhost:5173/callback"
  logoutUri: string;     // e.g., "http://localhost:5173"
  scope: string;         // e.g., "openid email profile"
}

function buildLoginUrl(config: CognitoConfig): string;
function buildLogoutUrl(config: CognitoConfig): string;
```

### Form Validation Hook

A custom hook provides reusable validation logic for all forms.

```typescript
// hooks/useValidation.ts
interface ValidationRule {
  validate: (value: unknown) => boolean;
  message: string;
}

interface FieldValidation {
  [fieldName: string]: ValidationRule[];
}

interface ValidationResult {
  isValid: boolean;
  errors: { [fieldName: string]: string };
}

function useValidation(rules: FieldValidation): {
  validate: (fieldName: string, value: unknown) => string | null;
  validateAll: (values: Record<string, unknown>) => ValidationResult;
  errors: { [fieldName: string]: string };
  clearError: (fieldName: string) => void;
};
```

**Validation Rules:**
- `required` - Field must have a value
- `minLength(n)` - String must be at least n characters
- `maxLength(n)` - String must be at most n characters
- `pattern(regex)` - String must match regex
- `pastDate` - Date must be in the past
- `notFutureDate` - Date must not be in the future
- `positiveInteger` - Number must be positive integer
- `dateRange` - End date must be after start date

### API Hooks

Custom hooks encapsulate API calls with loading and error states.

```typescript
// hooks/useApi.ts
interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

function useApi<T>(
  apiCall: () => Promise<ApiResponse<T>>,
  dependencies?: unknown[]
): ApiState<T> & {
  refetch: () => Promise<void>;
};

function useApiMutation<T, P>(
  apiCall: (params: P) => Promise<ApiResponse<T>>
): {
  mutate: (params: P) => Promise<ApiResponse<T>>;
  loading: boolean;
  error: string | null;
};
```

### Domain Types

TypeScript interfaces define the domain model matching the backend API.

```typescript
// types/domain.ts
interface Family {
  id: string;
  name: string;
  owner_id: string;
  created_at: string;
  updated_at: string;
}

interface Dependent {
  id: string;
  family_id: string;
  name: string;
  date_of_birth: string;
  created_at: string;
  updated_at: string;
}

type FeedingType = 'breast' | 'bottle' | 'solid';
type DiaperContents = 'wet' | 'dirty' | 'both';
type ActivityType = 'feeding' | 'diaper_change' | 'sleep' | 'pumping';

interface BaseActivity {
  id: string;
  dependent_id: string;
  activity_type: ActivityType;
  timestamp: string;
  created_at: string;
  updated_at: string;
}

interface FeedingActivity extends BaseActivity {
  activity_type: 'feeding';
  feeding_type: FeedingType;
}

interface DiaperActivity extends BaseActivity {
  activity_type: 'diaper_change';
  contents: DiaperContents;
}

interface SleepActivity extends BaseActivity {
  activity_type: 'sleep';
  start_time: string;
  end_time: string;
}

interface PumpingActivity extends BaseActivity {
  activity_type: 'pumping';
  volume_ml: number;
}

type Activity = FeedingActivity | DiaperActivity | SleepActivity | PumpingActivity;
```

## Data Models

### API Request/Response Models

```typescript
// api/types.ts

// Family endpoints
interface CreateFamilyRequest {
  name: string;
}

interface UpdateFamilyRequest {
  name: string;
}

interface FamilyResponse {
  id: string;
  name: string;
  owner_id: string;
  created_at: string;
  updated_at: string;
}

interface FamilyListResponse {
  families: FamilyResponse[];
}

// Dependent endpoints
interface CreateDependentRequest {
  family_id: string;
  name: string;
  date_of_birth: string; // ISO 8601 format
}

interface UpdateDependentRequest {
  name: string;
  date_of_birth: string;
}

interface DependentResponse {
  id: string;
  family_id: string;
  name: string;
  date_of_birth: string;
  created_at: string;
  updated_at: string;
}

// Activity endpoints
interface CreateActivityRequest {
  dependent_id: string;
  activity_type: ActivityType;
  timestamp: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
}

interface UpdateActivityRequest {
  timestamp: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
}

interface ActivityResponse {
  id: string;
  dependent_id: string;
  activity_type: ActivityType;
  timestamp: string;
  created_at: string;
  updated_at: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: number;
}

interface ActivityListResponse {
  activities: ActivityResponse[];
}

// Error response
interface ErrorResponse {
  error: string;
  details?: string[];
}
```

### Form State Models

```typescript
// Form state interfaces for controlled components
interface FamilyFormState {
  name: string;
}

interface DependentFormState {
  name: string;
  date_of_birth: string;
  family_id: string;
}

interface ActivityFormState {
  dependent_id: string;
  activity_type: ActivityType;
  timestamp: string;
  feeding_type?: FeedingType;
  contents?: DiaperContents;
  start_time?: string;
  end_time?: string;
  volume_ml?: string; // String for input, converted to number on submit
}
```

### Routing Configuration

```typescript
// App.tsx routing structure
const routes = [
  {
    path: '/',
    element: <ProtectedRoute><FamiliesPage /></ProtectedRoute>
  },
  {
    path: '/families/:familyId',
    element: <ProtectedRoute><FamilyDetailPage /></ProtectedRoute>
  },
  {
    path: '/dependents/:dependentId',
    element: <ProtectedRoute><DependentDetailPage /></ProtectedRoute>
  },
  {
    path: '/callback',
    element: <CallbackPage /> // Handles OAuth callback
  },
  {
    path: '*',
    element: <NotFoundPage />
  }
];
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Unauthenticated Access Redirects

*For any* application access attempt without a valid authentication token, the application should redirect to the Cognito login page.

**Validates: Requirements 1.2**

### Property 2: Token Storage

*For any* valid authentication token returned by Cognito, the application should store it securely in memory.

**Validates: Requirements 1.3**

### Property 3: Token Inclusion in API Requests

*For any* API request to the backend, the authentication token should be included in the Authorization header.

**Validates: Requirements 1.4, 16.2**

### Property 4: Expired Token Handling

*For any* API response with 401 status, the application should redirect the user to re-authenticate.

**Validates: Requirements 1.5**

### Property 5: Logout Clears Token

*For any* logout action, the authentication token should be cleared from storage, and subsequent checks should show no token present.

**Validates: Requirements 1.6**

### Property 6: Name Field Validation

*For any* form field representing a name (family name, dependent name), validation should reject strings with length less than 1 character.

**Validates: Requirements 2.2, 4.2, 6.2, 8.2**

### Property 7: Create Operations Use POST

*For any* create operation (family, dependent, activity), the API client should send a POST request to the appropriate endpoint.

**Validates: Requirements 2.3, 6.5, 10.9**

### Property 8: Successful Mutations Update UI

*For any* successful create or update operation, the application should display the new or updated data in the UI.

**Validates: Requirements 2.4, 4.4, 6.6, 8.5, 10.10, 12.4**

### Property 9: API Errors Display Messages

*For any* API error response, the application should display the error message to the user.

**Validates: Requirements 2.5, 4.5, 6.7, 8.6, 9.5, 10.11, 12.5, 13.5**

### Property 10: Loading State During Requests

*For any* API request in progress, the application should display a loading indicator.

**Validates: Requirements 2.6, 19.1**

### Property 11: Family List Rendering

*For any* set of families returned by the API, all families should appear in the rendered family list.

**Validates: Requirements 3.1**

### Property 12: Read Operations Use GET

*For any* read operation (fetching family, dependent, or activity details), the API client should send a GET request to the appropriate endpoint with the correct ID.

**Validates: Requirements 3.2, 7.2, 11.2, 11.7**

### Property 13: Family Data Completeness

*For any* family data response, the rendered output should include the family name, associated dependents, creation timestamp, and update timestamp.

**Validates: Requirements 3.3, 3.4**

### Property 14: Update Operations Use PUT

*For any* update operation (family, dependent, activity), the API client should send a PUT request to the appropriate endpoint with the correct ID.

**Validates: Requirements 4.3, 8.4, 12.3**

### Property 15: Deletion Requires Confirmation

*For any* delete action (family, dependent, activity), a confirmation dialog should be displayed before the deletion is executed.

**Validates: Requirements 5.2, 9.2, 13.2**

### Property 16: Delete Operations Use DELETE

*For any* delete operation (family, dependent, activity), the API client should send a DELETE request to the appropriate endpoint with the correct ID.

**Validates: Requirements 5.3, 9.3, 13.3**

### Property 17: Successful Deletion Removes from UI

*For any* successful delete operation, the deleted item should be removed from the UI display.

**Validates: Requirements 5.4, 9.4, 13.4**

### Property 18: Date of Birth Validation

*For any* date of birth field, validation should reject dates that are in the future.

**Validates: Requirements 6.3, 8.3**

### Property 19: Family ID Validation

*For any* dependent form, validation should reject submissions with missing or invalid family_id values.

**Validates: Requirements 6.4**

### Property 20: Dependent List Rendering

*For any* family view, all associated dependents should be displayed in the UI.

**Validates: Requirements 7.1**

### Property 21: Dependent Data Completeness

*For any* dependent data response, the rendered output should include the dependent name, date of birth, creation timestamp, and update timestamp.

**Validates: Requirements 7.3**

### Property 22: Age Calculation Accuracy

*For any* dependent with a date of birth, the displayed age should be correctly calculated based on the current date and the date of birth.

**Validates: Requirements 7.4**

### Property 23: Dependent ID Validation

*For any* activity form, validation should reject submissions with missing or invalid dependent_id values.

**Validates: Requirements 10.2**

### Property 24: Activity Type Validation

*For any* activity form, validation should reject activity types that are not one of: feeding, diaper_change, sleep, pumping.

**Validates: Requirements 10.3**

### Property 25: Timestamp Not in Future

*For any* activity timestamp field, validation should reject timestamps that are in the future.

**Validates: Requirements 10.4**

### Property 26: Feeding Type Conditional Validation

*For any* activity form where activity_type is "feeding", validation should require a feeding_type value that is one of: breast, bottle, solid.

**Validates: Requirements 10.5**

### Property 27: Diaper Contents Conditional Validation

*For any* activity form where activity_type is "diaper_change", validation should require a contents value that is one of: wet, dirty, both.

**Validates: Requirements 10.6**

### Property 28: Sleep Time Range Validation

*For any* activity form where activity_type is "sleep", validation should require start_time and end_time where end_time is after start_time.

**Validates: Requirements 10.7**

### Property 29: Pumping Volume Validation

*For any* activity form where activity_type is "pumping", validation should require volume_ml to be a positive integer.

**Validates: Requirements 10.8**

### Property 30: Activity Chronological Ordering

*For any* list of activities, they should be displayed in reverse chronological order (newest first) based on timestamp.

**Validates: Requirements 11.3**

### Property 31: Activity Data Completeness

*For any* activity data response, the rendered output should include the activity type, timestamp, and all type-specific details.

**Validates: Requirements 11.4, 11.8**

### Property 32: Date Range Filtering

*For any* date range filter applied to activities, only activities with timestamps within that range should be displayed.

**Validates: Requirements 11.5**

### Property 33: Activity Type Filtering

*For any* activity type filter applied to activities, only activities matching that type should be displayed.

**Validates: Requirements 11.6**

### Property 34: Browser History Maintenance

*For any* navigation action, the browser history should be updated to allow back/forward navigation.

**Validates: Requirements 14.4**

### Property 35: Invalid Route Handling

*For any* navigation to an unmatched route, the application should display a "Page not found" message.

**Validates: Requirements 14.5**

### Property 36: URL Preservation on Auth Redirect

*For any* authentication redirect, the current URL should be preserved and restored after successful authentication.

**Validates: Requirements 14.6**

### Property 37: Error Logging

*For any* error that occurs in the application, an entry should be logged to the browser console.

**Validates: Requirements 15.7**

### Property 38: URL Construction

*For any* API request path, the full URL should be correctly constructed by combining the configured base URL with the path.

**Validates: Requirements 16.1**

### Property 39: Content-Type Header for Body Requests

*For any* API request that includes a request body, the Content-Type header should be set to "application/json".

**Validates: Requirements 16.3**

### Property 40: JSON Response Parsing

*For any* JSON response from the backend API, the response should be correctly parsed into JavaScript objects.

**Validates: Requirements 16.4**

### Property 41: Error Message Extraction

*For any* error response from the backend API, the error message should be extracted and returned by the API client.

**Validates: Requirements 16.5**

### Property 42: Request Timeout Handling

*For any* API request that exceeds the configured timeout duration, the request should be aborted and a timeout error should be returned.

**Validates: Requirements 16.6**

### Property 43: Touch Target Minimum Size

*For any* interactive element (button, link, input), the rendered size should be at least 44x44 pixels to ensure usability on touch devices.

**Validates: Requirements 17.4**

### Property 44: Validation on Blur

*For any* form field, validation should be triggered when the user leaves the field (blur event).

**Validates: Requirements 18.1**

### Property 45: Validation Error Display

*For any* form field that fails validation, an error message should be displayed below the field.

**Validates: Requirements 18.2**

### Property 46: Validation Error Removal

*For any* form field that passes validation, any existing error message should be removed.

**Validates: Requirements 18.3**

### Property 47: Submit Button Disabled with Errors

*For any* form with validation errors, the submit button should be disabled.

**Validates: Requirements 18.4**

### Property 48: Required Field Indicators

*For any* required form field, a visual indicator should be present in the rendered output.

**Validates: Requirements 18.6**

### Property 49: Submit Button Disabled During Request

*For any* form with an API request in progress, the submit button should be disabled.

**Validates: Requirements 19.2**

### Property 50: Loading Placeholder Display

*For any* data loading state, a skeleton or placeholder UI should be displayed.

**Validates: Requirements 19.3**

### Property 51: Success Message Display

*For any* successful create, update, or delete operation, a success message should be displayed to the user.

**Validates: Requirements 19.4**

### Property 52: Success Message Auto-Dismiss

*For any* success message displayed, it should automatically disappear after 3 seconds.

**Validates: Requirements 19.5**

### Property 53: Base URL from Configuration

*For any* application startup, the backend API base URL should be read from configuration, not hardcoded.

**Validates: Requirements 20.1**

### Property 54: Cognito Configuration from Environment

*For any* application startup, the Cognito service configuration should be read from environment variables.

**Validates: Requirements 20.2**

### Property 55: Configuration Validation at Startup

*For any* required configuration value, the application should validate its presence at startup and fail to start if missing.

**Validates: Requirements 20.4**


## Error Handling

### Error Classification

The application handles errors at multiple levels with consistent patterns:

**1. Network Errors**
- Connection failures (no network, DNS failure, etc.)
- Timeout errors (request exceeds configured timeout)
- Display: "Connection failed. Please check your network and try again."

**2. HTTP Status Errors**
- 400 Bad Request: Display validation errors from response body
- 401 Unauthorized: Redirect to authentication, preserve current URL
- 403 Forbidden: Display "Access denied. You don't have permission to perform this action."
- 404 Not Found: Display context-specific message (e.g., "Family not found")
- 500 Internal Server Error: Display "Server error. Please try again later."

**3. Validation Errors**
- Client-side validation failures before submission
- Display field-level errors inline below each field
- Prevent form submission until all errors are resolved

**4. Application Errors**
- Unexpected JavaScript errors
- Caught by Error Boundary component
- Display fallback UI with error message and reload option

### Error Handling Architecture

```typescript
// utils/errorHandling.ts
interface ErrorInfo {
  message: string;
  type: 'network' | 'http' | 'validation' | 'application';
  statusCode?: number;
  details?: string[];
}

function parseApiError(error: unknown): ErrorInfo {
  // Network error
  if (error instanceof TypeError && error.message.includes('fetch')) {
    return {
      message: 'Connection failed. Please check your network and try again.',
      type: 'network'
    };
  }
  
  // Timeout error
  if (error instanceof Error && error.name === 'AbortError') {
    return {
      message: 'Request timed out. Please try again.',
      type: 'network'
    };
  }
  
  // HTTP error with response
  if (isHttpError(error)) {
    return parseHttpError(error);
  }
  
  // Unknown error
  return {
    message: 'An unexpected error occurred. Please try again.',
    type: 'application'
  };
}

function parseHttpError(error: HttpError): ErrorInfo {
  const statusCode = error.response.status;
  
  switch (statusCode) {
    case 400:
      return {
        message: error.response.data.error || 'Invalid request',
        type: 'validation',
        statusCode,
        details: error.response.data.details
      };
    case 401:
      return {
        message: 'Authentication required',
        type: 'http',
        statusCode
      };
    case 403:
      return {
        message: 'Access denied. You don\'t have permission to perform this action.',
        type: 'http',
        statusCode
      };
    case 404:
      return {
        message: 'Resource not found',
        type: 'http',
        statusCode
      };
    case 500:
      return {
        message: 'Server error. Please try again later.',
        type: 'http',
        statusCode
      };
    default:
      return {
        message: error.response.data.error || 'An error occurred',
        type: 'http',
        statusCode
      };
  }
}
```

### Error Boundary Component

```typescript
// components/common/ErrorBoundary.tsx
interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  ErrorBoundaryState
> {
  state = { hasError: false, error: null };
  
  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }
  
  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Application error:', error, errorInfo);
  }
  
  render() {
    if (this.state.hasError) {
      return (
        <div className="error-page">
          <h1>Something went wrong</h1>
          <p>The application encountered an unexpected error.</p>
          <button onClick={() => window.location.reload()}>
            Reload Page
          </button>
        </div>
      );
    }
    
    return this.props.children;
  }
}
```

### API Client Error Handling

The API client implements consistent error handling for all requests:

```typescript
// api/client.ts
async function request<T>(
  method: string,
  path: string,
  body?: unknown
): Promise<ApiResponse<T>> {
  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);
    
    const response = await fetch(`${this.config.baseURL}${path}`, {
      method,
      headers: await this.buildHeaders(body !== undefined),
      body: body ? JSON.stringify(body) : undefined,
      signal: controller.signal
    });
    
    clearTimeout(timeoutId);
    
    // Handle 401 specially - trigger re-authentication
    if (response.status === 401) {
      // Trigger auth redirect (handled by auth context)
      window.dispatchEvent(new CustomEvent('auth:expired'));
      return { error: 'Authentication required' };
    }
    
    // Parse response
    const data = await response.json();
    
    if (!response.ok) {
      console.error(`API error ${response.status}:`, data);
      return { error: data.error || `Request failed with status ${response.status}` };
    }
    
    return { data };
    
  } catch (error) {
    console.error('Request failed:', error);
    const errorInfo = parseApiError(error);
    return { error: errorInfo.message };
  }
}
```

### Form Validation Error Display

Form components use react-bootstrap components with validation states:

```typescript
// components/common/Input.tsx
import { Form } from 'react-bootstrap';

interface InputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  onBlur?: () => void;
  error?: string;
  required?: boolean;
  type?: string;
}

function Input({ label, value, onChange, onBlur, error, required, type = 'text' }: InputProps) {
  return (
    <Form.Group className="mb-3">
      <Form.Label>
        {label}
        {required && <span className="text-danger ms-1">*</span>}
      </Form.Label>
      <Form.Control
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={onBlur}
        isInvalid={!!error}
        aria-invalid={!!error}
        aria-describedby={error ? `${label}-error` : undefined}
      />
      {error && (
        <Form.Control.Feedback type="invalid" id={`${label}-error`}>
          {error}
        </Form.Control.Feedback>
      )}
    </Form.Group>
  );
}
```

**react-bootstrap Components Used:**
- `Form.Group` - Wraps form fields with consistent spacing
- `Form.Label` - Accessible labels with Bootstrap styling
- `Form.Control` - Styled input fields with validation states
- `Form.Control.Feedback` - Validation error messages
- `Button` - Styled buttons with variants (primary, secondary, danger)
- `Modal` - Confirmation dialogs for delete operations
- `Card` - Display families, dependents, and activities
- `Container`, `Row`, `Col` - Responsive grid layout
- `Spinner` - Loading indicators
- `Alert` - Success and error messages
- `ListGroup` - Lists of items

### User Feedback for Errors

All errors are logged to the console for debugging while displaying user-friendly messages:

```typescript
// hooks/useApi.ts
function useApiMutation<T, P>(
  apiCall: (params: P) => Promise<ApiResponse<T>>
) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const mutate = async (params: P) => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await apiCall(params);
      
      if (response.error) {
        setError(response.error);
        console.error('API mutation error:', response.error);
      }
      
      return response;
    } finally {
      setLoading(false);
    }
  };
  
  return { mutate, loading, error };
}
```

## Testing Strategy

### Testing Approach

The application uses a dual testing approach combining unit tests and property-based tests:

**Unit Tests:**
- Specific examples demonstrating correct behavior
- Edge cases and boundary conditions
- Integration points between components
- Error handling scenarios

**Property-Based Tests:**
- Universal properties that hold for all inputs
- Comprehensive input coverage through randomization
- Validation logic across input ranges
- API client behavior across different scenarios

Both approaches are complementary and necessary for comprehensive coverage. Unit tests catch concrete bugs and document expected behavior, while property tests verify general correctness across a wide range of inputs.

### Testing Tools

**Test Framework:** Vitest
- Fast, Vite-native test runner
- Compatible with Jest API
- Built-in coverage reporting

**Property-Based Testing:** fast-check
- JavaScript/TypeScript property-based testing library
- Generates random test cases
- Shrinks failing cases to minimal examples

**React Testing:** @testing-library/react
- User-centric testing approach
- Tests components as users interact with them
- Encourages accessible markup

**Mocking:** Vitest built-in mocks
- Mock API calls
- Mock authentication
- Mock browser APIs (localStorage, fetch)

### Test Organization

```
src/
├── api/
│   ├── client.test.ts           # API client unit tests
│   ├── client.property.test.ts  # API client property tests
│   ├── families.test.ts
│   └── activities.test.ts
├── components/
│   ├── common/
│   │   ├── Button.test.tsx
│   │   ├── Input.test.tsx
│   │   └── Input.property.test.tsx
│   ├── families/
│   │   ├── FamilyForm.test.tsx
│   │   └── FamilyForm.property.test.tsx
│   └── activities/
│       ├── ActivityForm.test.tsx
│       └── ActivityForm.property.test.tsx
├── hooks/
│   ├── useValidation.test.ts
│   └── useValidation.property.test.ts
└── utils/
    ├── validation.test.ts
    └── validation.property.test.ts
```

### Property-Based Test Configuration

All property-based tests must:
- Run minimum 100 iterations (configured via fast-check)
- Include a comment tag referencing the design property
- Use descriptive test names

**Tag Format:**
```typescript
// Feature: famtrac-frontend, Property 6: Name Field Validation
test('name validation rejects strings shorter than 1 character', () => {
  fc.assert(
    fc.property(fc.string(), (name) => {
      if (name.length < 1) {
        const result = validateName(name);
        expect(result.isValid).toBe(false);
      }
    }),
    { numRuns: 100 }
  );
});
```

### Test Coverage Requirements

**Validation Logic (utils/validation.ts):**
- Property tests for all validation rules
- Test that valid inputs pass
- Test that invalid inputs fail with correct messages
- Edge cases: empty strings, whitespace, boundary values

**API Client (api/client.ts):**
- Property test: all requests include auth token
- Property test: all requests with body include Content-Type header
- Property test: URL construction combines base URL and path correctly
- Unit tests: timeout handling, error parsing, 401 handling
- Unit tests: specific HTTP status codes (400, 403, 404, 500)

**Form Components:**
- Property test: validation triggers on blur for all fields
- Property test: error messages display for all validation failures
- Property test: submit button disabled when errors exist
- Unit tests: specific form submissions (create family, add dependent)
- Unit tests: form reset after successful submission

**Authentication (auth/):**
- Unit test: redirect to login when no token
- Unit test: token stored after successful login
- Unit test: logout clears token
- Unit test: 401 response triggers re-authentication
- Property test: all API calls include token

**Routing (App.tsx):**
- Unit tests: each route renders correct component
- Unit test: invalid routes show 404 page
- Unit test: protected routes redirect when not authenticated
- Property test: navigation updates browser history

**Activity Forms:**
- Property test: conditional validation for each activity type
- Property test: feeding activities require feeding_type
- Property test: diaper activities require contents
- Property test: sleep activities require valid time range
- Property test: pumping activities require positive volume
- Unit tests: specific activity creation examples

**Date Utilities (utils/dateUtils.ts):**
- Property test: age calculation accuracy for all dates
- Unit tests: specific date formatting examples
- Edge cases: leap years, timezone handling

**Error Handling:**
- Unit tests: each HTTP status code produces correct message
- Unit tests: network errors produce correct message
- Unit tests: timeout errors produce correct message
- Property test: all errors logged to console

### Example Property-Based Tests

**Validation Property Test:**
```typescript
// Feature: famtrac-frontend, Property 6: Name Field Validation
import fc from 'fast-check';
import { validateName } from './validation';

describe('Name Validation Properties', () => {
  test('rejects names shorter than 1 character', () => {
    fc.assert(
      fc.property(fc.string(), (name) => {
        if (name.length < 1) {
          const result = validateName(name);
          expect(result.isValid).toBe(false);
          expect(result.error).toBeTruthy();
        }
      }),
      { numRuns: 100 }
    );
  });
  
  test('accepts names with 1 or more characters', () => {
    fc.assert(
      fc.property(
        fc.string({ minLength: 1 }),
        (name) => {
          const result = validateName(name);
          expect(result.isValid).toBe(true);
          expect(result.error).toBeNull();
        }
      ),
      { numRuns: 100 }
    );
  });
});
```

**API Client Property Test:**
```typescript
// Feature: famtrac-frontend, Property 3: Token Inclusion in API Requests
import fc from 'fast-check';
import { ApiClient } from './client';

describe('API Client Properties', () => {
  test('includes auth token in all requests', async () => {
    const mockToken = 'test-token-123';
    const mockGetToken = jest.fn().mockResolvedValue(mockToken);
    const client = new ApiClient(config, mockGetToken);
    
    await fc.assert(
      fc.asyncProperty(
        fc.string({ minLength: 1 }), // Random path
        async (path) => {
          // Mock fetch to capture headers
          const mockFetch = jest.fn().mockResolvedValue({
            ok: true,
            json: async () => ({})
          });
          global.fetch = mockFetch;
          
          await client.get(path);
          
          const callArgs = mockFetch.mock.calls[0];
          const headers = callArgs[1].headers;
          expect(headers.Authorization).toBe(`Bearer ${mockToken}`);
        }
      ),
      { numRuns: 100 }
    );
  });
});
```

**Activity Sorting Property Test:**
```typescript
// Feature: famtrac-frontend, Property 30: Activity Chronological Ordering
import fc from 'fast-check';
import { sortActivitiesByTimestamp } from './activities';

describe('Activity Sorting Properties', () => {
  test('sorts activities in reverse chronological order', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.record({
            id: fc.uuid(),
            timestamp: fc.date().map(d => d.toISOString()),
            activity_type: fc.constantFrom('feeding', 'diaper_change', 'sleep', 'pumping')
          })
        ),
        (activities) => {
          const sorted = sortActivitiesByTimestamp(activities);
          
          // Verify each activity is newer than or equal to the next
          for (let i = 0; i < sorted.length - 1; i++) {
            const current = new Date(sorted[i].timestamp);
            const next = new Date(sorted[i + 1].timestamp);
            expect(current.getTime()).toBeGreaterThanOrEqual(next.getTime());
          }
        }
      ),
      { numRuns: 100 }
    );
  });
});
```

### Integration Testing

While the focus is on unit and property tests, key integration points should be tested:

**Authentication Flow:**
- Test complete login flow (mock Cognito)
- Test token refresh on expiration
- Test logout and re-login

**CRUD Operations:**
- Test complete create-read-update-delete flow for each entity
- Test cascading effects (e.g., deleting family with dependents)
- Test error recovery

**Form Submission:**
- Test complete form validation and submission flow
- Test error display and correction
- Test success feedback

### Continuous Integration

Tests run automatically on:
- Every commit (pre-commit hook)
- Every pull request
- Before deployment

**CI Configuration:**
```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm ci
      - run: npm run test
      - run: npm run test:coverage
      - uses: codecov/codecov-action@v3
```

**Coverage Goals:**
- Overall: 80% minimum
- Validation logic: 95% minimum
- API client: 90% minimum
- Critical paths (auth, CRUD): 90% minimum

