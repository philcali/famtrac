import { Container, Button, Alert } from 'react-bootstrap';
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
    <Container
      className="d-flex flex-column align-items-center justify-content-center"
      style={{ minHeight: '100vh' }}
    >
      <div className="text-center" style={{ maxWidth: '600px' }}>
        <h1 className="display-4 fw-bold text-danger mb-4">Something went wrong</h1>
        <Alert variant="danger" className="text-start">
          <Alert.Heading>An unexpected error occurred</Alert.Heading>
          {error && (
            <p className="mb-0">
              <small className="font-monospace">{error.message}</small>
            </p>
          )}
        </Alert>
        <p className="text-muted mb-4">
          We're sorry for the inconvenience. Please try reloading the page or returning to the home
          page.
        </p>
        <div className="d-flex gap-2 justify-content-center">
          <Button variant="primary" onClick={handleGoHome}>
            Go to Home
          </Button>
          <Button variant="outline-secondary" onClick={handleReload}>
            Reload Page
          </Button>
        </div>
      </div>
    </Container>
  );
}
