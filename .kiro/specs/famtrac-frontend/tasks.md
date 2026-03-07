# Implementation Plan: famtrac-frontend

## Overview

This plan implements a modern single-page application (SPA) using Vite, React 18, TypeScript, and react-bootstrap. The application integrates with the famtrac-backend REST API and AWS Cognito for authentication. Implementation follows an incremental approach, building core infrastructure first, then adding features layer by layer with testing integrated throughout.

## Tasks

- [x] 1. Project setup and configuration
  - [x] 1.1 Initialize Vite project with React and TypeScript
    - Create project using `npm create vite@latest famtrac-frontend -- --template react-ts`
    - Install dependencies: react-router-dom, react-bootstrap, bootstrap, fast-check, @testing-library/react, @testing-library/user-event, @testing-library/jest-dom
    - Configure TypeScript with strict mode enabled
    - Set up ESLint and Prettier for code quality
    - _Requirements: 20.1, 20.2, 20.3_
  
  - [x] 1.2 Create environment configuration
    - Create `.env.development` and `.env.production` files
    - Define environment variables for API base URL and Cognito configuration
    - Implement `src/config/environment.ts` to read and validate configuration
    - Implement `src/config/cognito.ts` with Cognito OAuth configuration
    - _Requirements: 20.1, 20.2, 20.4, 20.5_
  
  - [ ]* 1.3 Write property test for configuration validation
    - **Property 53: Base URL from Configuration**
    - **Property 54: Cognito Configuration from Environment**
    - **Property 55: Configuration Validation at Startup**
    - **Validates: Requirements 20.1, 20.2, 20.4**

- [x] 2. Core utilities and validation
  - [x] 2.1 Implement validation utilities
    - Create `src/utils/validation.ts` with validation functions
    - Implement rules: required, minLength, maxLength, pattern, pastDate, notFutureDate, positiveInteger, dateRange
    - _Requirements: 2.2, 4.2, 6.2, 6.3, 8.2, 8.3, 10.4, 10.5, 10.6, 10.7, 10.8, 18.1_
  
  - [ ]* 2.2 Write property tests for validation rules
    - **Property 6: Name Field Validation**
    - **Property 18: Date of Birth Validation**
    - **Property 25: Timestamp Not in Future**
    - **Property 29: Pumping Volume Validation**
    - **Validates: Requirements 2.2, 4.2, 6.2, 6.3, 8.2, 8.3, 10.4, 10.8**
  
  - [x] 2.3 Implement date utilities
    - Create `src/utils/dateUtils.ts` with date formatting and age calculation
    - Implement age calculation based on date of birth
    - _Requirements: 7.4_
  
  - [ ]* 2.4 Write property test for age calculation
    - **Property 22: Age Calculation Accuracy**
    - **Validates: Requirements 7.4**
  
  - [x] 2.5 Implement error handling utilities
    - Create `src/utils/errorHandling.ts` with error parsing functions
    - Implement parseApiError and parseHttpError functions
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7_

- [x] 3. API client implementation
  - [x] 3.1 Create API client core
    - Create `src/api/client.ts` with ApiClient class
    - Implement HTTP methods: get, post, put, delete
    - Implement request timeout handling with AbortController
    - Implement error handling and response parsing
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5, 16.6, 16.7_
  
  - [ ]* 3.2 Write property tests for API client
    - **Property 3: Token Inclusion in API Requests**
    - **Property 38: URL Construction**
    - **Property 39: Content-Type Header for Body Requests**
    - **Property 40: JSON Response Parsing**
    - **Property 41: Error Message Extraction**
    - **Property 42: Request Timeout Handling**
    - **Validates: Requirements 1.4, 16.1, 16.2, 16.3, 16.4, 16.5, 16.6**
  
  - [x] 3.3 Implement API type definitions
    - Create `src/api/types.ts` with request/response interfaces
    - Define types for Family, Dependent, and Activity endpoints
    - _Requirements: 2.3, 3.2, 4.3, 5.3, 6.5, 7.2, 8.4, 9.3, 10.9, 11.2, 12.3, 13.3_
  
  - [x] 3.4 Implement domain type definitions
    - Create `src/types/domain.ts` with domain model interfaces
    - Define Family, Dependent, Activity types and enums
    - _Requirements: 2.4, 3.3, 6.6, 7.3, 10.10, 11.4_

- [x] 4. Authentication system
  - [x] 4.1 Implement token service
    - Create `src/auth/tokenService.ts` with token management functions
    - Implement parseTokensFromUrl, storeTokens, getAccessToken, clearTokens
    - Implement isTokenExpired and refreshAccessToken functions
    - Store tokens in sessionStorage for security
    - _Requirements: 1.2, 1.3, 1.5, 1.6_
  
  - [ ]* 4.2 Write property tests for token handling
    - **Property 2: Token Storage**
    - **Property 5: Logout Clears Token**
    - **Validates: Requirements 1.3, 1.6**
  
  - [x] 4.3 Implement authentication context
    - Create `src/auth/AuthProvider.tsx` with React Context
    - Implement useAuth hook in `src/auth/useAuth.ts`
    - Provide login, logout, getToken, and authentication state
    - Handle OAuth callback flow and token refresh
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_
  
  - [x] 4.4 Implement protected route component
    - Create `src/auth/ProtectedRoute.tsx` for route guarding
    - Redirect to Cognito login when not authenticated
    - Preserve current URL for post-login redirect
    - _Requirements: 1.2, 14.6_
  
  - [ ]* 4.5 Write property tests for authentication flow
    - **Property 1: Unauthenticated Access Redirects**
    - **Property 4: Expired Token Handling**
    - **Property 36: URL Preservation on Auth Redirect**
    - **Validates: Requirements 1.2, 1.5, 14.6**

- [ ] 5. Checkpoint - Ensure core infrastructure works
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Common UI components
  - [ ] 6.1 Create base form components
    - Create `src/components/common/Input.tsx` with validation support
    - Create `src/components/common/Button.tsx` with loading states
    - Use react-bootstrap Form components with accessibility features
    - _Requirements: 17.4, 18.2, 18.3, 18.6, 19.2_
  
  - [ ]* 6.2 Write property tests for form components
    - **Property 43: Touch Target Minimum Size**
    - **Property 44: Validation on Blur**
    - **Property 45: Validation Error Display**
    - **Property 46: Validation Error Removal**
    - **Property 48: Required Field Indicators**
    - **Validates: Requirements 17.4, 18.1, 18.2, 18.3, 18.6**
  
  - [ ] 6.3 Create feedback components
    - Create `src/components/common/LoadingSpinner.tsx` using react-bootstrap Spinner
    - Create `src/components/common/ErrorMessage.tsx` using react-bootstrap Alert
    - Create `src/components/common/SuccessMessage.tsx` with auto-dismiss
    - Create `src/components/common/ConfirmDialog.tsx` using react-bootstrap Modal
    - _Requirements: 2.6, 5.2, 9.2, 13.2, 19.1, 19.4, 19.5_
  
  - [ ]* 6.4 Write property tests for feedback components
    - **Property 10: Loading State During Requests**
    - **Property 51: Success Message Display**
    - **Property 52: Success Message Auto-Dismiss**
    - **Validates: Requirements 2.6, 19.1, 19.4, 19.5**
  
  - [ ] 6.5 Create error boundary component
    - Create `src/components/common/ErrorBoundary.tsx` for application errors
    - Display fallback UI with reload option
    - Log errors to console
    - _Requirements: 15.7_

- [ ] 7. Custom hooks for API and forms
  - [ ] 7.1 Implement useApi hook
    - Create `src/hooks/useApi.ts` for data fetching with loading/error states
    - Implement useApiMutation for create/update/delete operations
    - _Requirements: 2.5, 2.6, 4.5, 6.7, 8.6, 9.5, 10.11, 12.5, 13.5_
  
  - [ ] 7.2 Implement useValidation hook
    - Create `src/hooks/useValidation.ts` for form validation logic
    - Support field-level and form-level validation
    - Track validation errors and provide clear/validate functions
    - _Requirements: 18.1, 18.2, 18.3, 18.4_
  
  - [ ]* 7.3 Write property tests for validation hook
    - **Property 47: Submit Button Disabled with Errors**
    - **Validates: Requirements 18.4**
  
  - [ ] 7.4 Implement useForm hook
    - Create `src/hooks/useForm.ts` for form state management
    - Integrate with useValidation for automatic validation
    - _Requirements: 18.1, 18.4_

- [ ] 8. Family management features
  - [ ] 8.1 Implement family API methods
    - Create `src/api/families.ts` with CRUD operations
    - Implement getFamilies, getFamily, createFamily, updateFamily, deleteFamily
    - _Requirements: 2.3, 3.1, 3.2, 4.3, 5.3_
  
  - [ ]* 8.2 Write property tests for family API operations
    - **Property 7: Create Operations Use POST**
    - **Property 12: Read Operations Use GET**
    - **Property 14: Update Operations Use PUT**
    - **Property 16: Delete Operations Use DELETE**
    - **Validates: Requirements 2.3, 3.2, 4.3, 5.3**
  
  - [ ] 8.3 Create family UI components
    - Create `src/components/families/FamilyList.tsx` to display all families
    - Create `src/components/families/FamilyCard.tsx` using react-bootstrap Card
    - Create `src/components/families/FamilyForm.tsx` for create/edit
    - _Requirements: 2.1, 2.2, 3.1, 3.3, 3.4, 4.1, 4.2, 5.1_
  
  - [ ]* 8.4 Write property tests for family components
    - **Property 8: Successful Mutations Update UI**
    - **Property 9: API Errors Display Messages**
    - **Property 11: Family List Rendering**
    - **Property 13: Family Data Completeness**
    - **Property 15: Deletion Requires Confirmation**
    - **Property 17: Successful Deletion Removes from UI**
    - **Validates: Requirements 2.4, 2.5, 3.1, 3.3, 3.4, 5.2, 5.4**
  
  - [ ] 8.5 Create families page
    - Create `src/pages/FamiliesPage.tsx` as main families list view
    - Integrate FamilyList and FamilyForm components
    - Handle create, update, delete operations with user feedback
    - _Requirements: 2.1, 2.4, 2.5, 3.1, 4.1, 4.4, 5.1, 5.4, 14.1_

- [ ] 9. Dependent management features
  - [ ] 9.1 Implement dependent API methods
    - Create `src/api/dependents.ts` with CRUD operations
    - Implement getDependents, getDependent, createDependent, updateDependent, deleteDependent
    - _Requirements: 6.5, 7.1, 7.2, 8.4, 9.3_
  
  - [ ] 9.2 Create dependent UI components
    - Create `src/components/dependents/DependentList.tsx` to display dependents
    - Create `src/components/dependents/DependentCard.tsx` using react-bootstrap Card
    - Create `src/components/dependents/DependentForm.tsx` for create/edit
    - Display age calculation in dependent cards
    - _Requirements: 6.1, 6.2, 6.3, 7.1, 7.3, 7.4, 8.1, 8.2, 9.1_
  
  - [ ]* 9.3 Write property tests for dependent components
    - **Property 19: Family ID Validation**
    - **Property 20: Dependent List Rendering**
    - **Property 21: Dependent Data Completeness**
    - **Validates: Requirements 6.4, 7.1, 7.3**
  
  - [ ] 9.4 Create family detail page
    - Create `src/pages/FamilyDetailPage.tsx` to view single family
    - Display family information and associated dependents
    - Support adding, editing, and deleting dependents
    - _Requirements: 3.2, 3.3, 6.1, 6.6, 7.1, 8.1, 8.5, 9.1, 9.4, 14.2_
  
  - [ ] 9.5 Create dependent detail page
    - Create `src/pages/DependentDetailPage.tsx` to view single dependent
    - Display dependent information with age
    - Prepare for activity list integration (next phase)
    - _Requirements: 7.2, 7.3, 7.4, 8.1, 8.5, 14.3_

- [ ] 10. Checkpoint - Ensure family and dependent features work
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 11. Activity management features
  - [ ] 11.1 Implement activity API methods
    - Create `src/api/activities.ts` with CRUD operations
    - Implement getActivities, getActivity, createActivity, updateActivity, deleteActivity
    - Support filtering by date range and activity type
    - _Requirements: 10.9, 11.2, 11.5, 11.6, 11.7, 12.3, 13.3_
  
  - [ ]* 11.2 Write property tests for activity API operations
    - **Property 23: Dependent ID Validation**
    - **Property 24: Activity Type Validation**
    - **Validates: Requirements 10.2, 10.3**
  
  - [ ] 11.3 Create activity form component
    - Create `src/components/activities/ActivityForm.tsx` for create/edit
    - Implement conditional validation based on activity type
    - Show/hide fields based on selected activity type
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 10.7, 10.8, 12.1, 12.2_
  
  - [ ]* 11.4 Write property tests for activity form validation
    - **Property 26: Feeding Type Conditional Validation**
    - **Property 27: Diaper Contents Conditional Validation**
    - **Property 28: Sleep Time Range Validation**
    - **Validates: Requirements 10.5, 10.6, 10.7**
  
  - [ ] 11.5 Create activity display components
    - Create `src/components/activities/ActivityCard.tsx` using react-bootstrap Card
    - Create `src/components/activities/ActivityList.tsx` with sorting
    - Create `src/components/activities/ActivityFilters.tsx` for date/type filtering
    - Implement reverse chronological sorting
    - _Requirements: 11.1, 11.3, 11.4, 11.5, 11.6_
  
  - [ ]* 11.6 Write property tests for activity display
    - **Property 30: Activity Chronological Ordering**
    - **Property 31: Activity Data Completeness**
    - **Property 32: Date Range Filtering**
    - **Property 33: Activity Type Filtering**
    - **Validates: Requirements 11.3, 11.4, 11.5, 11.6**
  
  - [ ] 11.7 Integrate activities into dependent detail page
    - Update DependentDetailPage to display activity list
    - Add activity creation, editing, and deletion
    - Implement filtering controls
    - _Requirements: 10.1, 10.10, 11.1, 11.2, 11.8, 12.1, 12.4, 13.1, 13.4_

- [ ] 12. Routing and navigation
  - [ ] 12.1 Set up React Router
    - Create `src/App.tsx` with route configuration
    - Define routes for families list, family detail, dependent detail
    - Add callback route for OAuth handling
    - Add 404 not found route
    - _Requirements: 14.1, 14.2, 14.3, 14.5_
  
  - [ ]* 12.2 Write property tests for routing
    - **Property 34: Browser History Maintenance**
    - **Property 35: Invalid Route Handling**
    - **Validates: Requirements 14.4, 14.5**
  
  - [ ] 12.3 Create navigation components
    - Add navigation bar using react-bootstrap Navbar
    - Include logout button in navigation
    - Show current user information
    - _Requirements: 1.6, 14.1_
  
  - [ ] 12.4 Create error pages
    - Create `src/pages/NotFoundPage.tsx` for 404 errors
    - Create `src/pages/ErrorPage.tsx` for general errors
    - _Requirements: 14.5, 15.4_

- [ ] 13. Responsive design and styling
  - [ ] 13.1 Set up Bootstrap and custom styles
    - Import Bootstrap CSS in main.tsx
    - Create `src/styles/custom.css` for custom styling
    - Configure responsive breakpoints
    - _Requirements: 17.1, 17.2, 17.3_
  
  - [ ] 13.2 Implement responsive layouts
    - Use react-bootstrap Container, Row, Col for grid layouts
    - Implement mobile-first responsive design
    - Ensure touch targets meet minimum size requirements
    - Test layouts at various screen widths (320px to 1920px)
    - _Requirements: 17.1, 17.2, 17.3, 17.4, 17.5_
  
  - [ ] 13.3 Add loading states and placeholders
    - Implement skeleton loading UI for data fetching
    - Add loading spinners for form submissions
    - Disable buttons during API requests
    - _Requirements: 19.1, 19.2, 19.3_
  
  - [ ]* 13.4 Write property tests for loading states
    - **Property 49: Submit Button Disabled During Request**
    - **Property 50: Loading Placeholder Display**
    - **Validates: Requirements 19.2, 19.3**

- [ ] 14. Error handling integration
  - [ ] 14.1 Implement global error handling
    - Add ErrorBoundary to App.tsx
    - Set up global error event listener for auth expiration
    - Ensure all errors are logged to console
    - _Requirements: 15.7_
  
  - [ ]* 14.2 Write property test for error logging
    - **Property 37: Error Logging**
    - **Validates: Requirements 15.7**
  
  - [ ] 14.3 Add HTTP status-specific error handling
    - Implement error message mapping for each status code
    - Add 401 handling with auth redirect
    - Add 403, 404, 500 error displays
    - Handle network errors and timeouts
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6_

- [ ] 15. Final integration and polish
  - [ ] 15.1 Create application entry point
    - Create `src/main.tsx` with React root and providers
    - Wrap app with AuthProvider and ErrorBoundary
    - Import Bootstrap CSS and custom styles
    - _Requirements: 1.1, 20.1, 20.2_
  
  - [ ] 15.2 Add OAuth callback handling
    - Create `src/pages/CallbackPage.tsx` to handle OAuth redirect
    - Parse tokens from URL and store them
    - Redirect to preserved URL or home page
    - _Requirements: 1.2, 1.3, 14.6_
  
  - [ ] 15.3 Configure Vite build settings
    - Update `vite.config.ts` with proper build configuration
    - Configure environment variable handling
    - Set up proxy for development API calls if needed
    - _Requirements: 20.3_
  
  - [ ] 15.4 Add comprehensive error handling tests
    - Test each HTTP status code produces correct error message
    - Test network error handling
    - Test timeout error handling
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6_

- [ ] 16. Final checkpoint - Complete testing and validation
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation at key milestones
- Property tests validate universal correctness properties using fast-check
- Unit tests validate specific examples and edge cases
- The implementation uses TypeScript throughout for type safety
- react-bootstrap provides accessible, responsive UI components
- Custom Cognito integration avoids heavy dependencies like AWS Amplify
- Simple state management with React Context keeps the application lightweight
