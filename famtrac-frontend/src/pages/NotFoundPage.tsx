import { Button } from '../components/common/Button';
import { useNavigate } from 'react-router-dom';

/**
 * NotFoundPage - 404 error page
 * Displays when user navigates to an invalid route (Requirement 14.5)
 * Provides navigation back to home page (Requirement 15.4)
 */
export function NotFoundPage() {
  const navigate = useNavigate();

  return (
    <div className="flex flex-col items-center justify-center min-h-screen px-4">
      <div className="text-center">
        <h1 className="text-7xl font-bold text-blue-600">404</h1>
        <h2 className="text-xl font-semibold mb-4">Page not found</h2>
        <p className="text-muted mb-4">
          The page you're looking for doesn't exist or has been moved.
        </p>
        <Button variant="primary" onClick={() => navigate('/')}>
          Go to Home
        </Button>
      </div>
    </div>
  );
}
