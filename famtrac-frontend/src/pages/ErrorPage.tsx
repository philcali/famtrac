import { Button } from '../components/common/Button';
import { useNavigate } from 'react-router-dom';

interface ErrorPageProps {
  error?: Error;
  resetError?: () => void;
}

/**
 * ErrorPage - General error page
 * Displays when an unexpected error occurs (Requirement 15.4)
 * Can be used by ErrorBoundary or as a standalone page
 */
export function ErrorPage({ error, resetError }: ErrorPageProps) {
  const navigate = useNavigate();

  const handleGoHome = () => {
    if (resetError) {
      resetError();
    }
    navigate('/');
  };

  const handleReload = () => {
    window.location.reload();
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen px-4">
      <div className="text-center" style={{ maxWidth: '600px' }}>
        <h1 className="text-4xl font-bold text-red-500 mb-4">Something went wrong</h1>
        <div className="text-left p-4 bg-red-50 border border-red-100 rounded-xl mb-4">
          <h2 className="text-base font-semibold mb-2">An unexpected error occurred</h2>
          {error && (
            <p className="mb-0">
              <small className="font-mono text-sm">{error.message}</small>
            </p>
          )}
        </div>
        <p className="text-muted mb-4">
          We're sorry for the inconvenience. Please try reloading the page or returning to the home
          page.
        </p>
        <div className="flex gap-2 justify-center">
          <Button variant="primary" onClick={handleGoHome}>
            Go to Home
          </Button>
          <Button variant="secondary" onClick={handleReload}>
            Reload Page
          </Button>
        </div>
      </div>
    </div>
  );
}
