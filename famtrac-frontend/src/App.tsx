import { Routes, Route } from 'react-router-dom';
import { FamiliesPage } from './pages/FamiliesPage';
import { FamilyDetailPage } from './pages/FamilyDetailPage';
import { DependentDetailPage } from './pages/DependentDetailPage';
import { ErrorBoundary } from './components/common/ErrorBoundary';
import './App.css';

/**
 * App component - Main application with routing
 * - Defines routes for families list, family detail, and dependent detail (Requirements 14.1, 14.2, 14.3)
 * - Wraps application in ErrorBoundary for error handling (Requirement 15.7)
 */
function App() {
  return (
    <ErrorBoundary>
      <Routes>
        <Route path="/" element={<FamiliesPage />} />
        <Route path="/families/:familyId" element={<FamilyDetailPage />} />
        <Route path="/dependents/:dependentId" element={<DependentDetailPage />} />
      </Routes>
    </ErrorBoundary>
  );
}

export default App;
