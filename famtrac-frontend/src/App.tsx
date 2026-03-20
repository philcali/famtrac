import { Routes, Route } from 'react-router-dom';
import { FamiliesPage } from './pages/FamiliesPage';
import { FamilyDetailPage } from './pages/FamilyDetailPage';
import { DependentDetailPage } from './pages/DependentDetailPage';
import { PendingSharesPage } from './pages/PendingSharesPage';
import { CallbackPage } from './pages/CallbackPage';
import { NotFoundPage } from './pages/NotFoundPage';
import { ProtectedRoute } from './auth/ProtectedRoute';
import { ErrorBoundary } from './components/common/ErrorBoundary';
import { Navigation } from './components/common/Navigation';
import './App.css';

/**
 * App component - Main application with routing
 * - Defines routes for families list, family detail, and dependent detail (Requirements 14.1, 14.2, 14.3)
 * - Adds callback route for OAuth handling (Requirement 14.1)
 * - Adds 404 not found route (Requirement 14.5)
 * - Wraps protected routes with ProtectedRoute component (Requirement 1.2)
 * - Includes navigation bar with logout (Requirements 1.6, 14.1)
 * - Wraps application in ErrorBoundary for error handling (Requirement 15.7)
 */
function App() {
  return (
    <ErrorBoundary>
      <Navigation />
      <Routes>
        {/* Protected routes requiring authentication */}
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <FamiliesPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/families/:familyId"
          element={
            <ProtectedRoute>
              <FamilyDetailPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/families/:familyId/dependents/:dependentId"
          element={
            <ProtectedRoute>
              <DependentDetailPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/shares"
          element={
            <ProtectedRoute>
              <PendingSharesPage />
            </ProtectedRoute>
          }
        />

        {/* Public routes */}
        <Route path="/login" element={<CallbackPage />} />

        {/* 404 catch-all route */}
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </ErrorBoundary>
  );
}

export default App;
