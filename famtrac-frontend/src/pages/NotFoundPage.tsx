import { Container, Button } from 'react-bootstrap';
import { useNavigate } from 'react-router-dom';

/**
 * NotFoundPage - 404 error page
 * Displays when user navigates to an invalid route (Requirement 14.5)
 * Provides navigation back to home page (Requirement 15.4)
 */
export function NotFoundPage() {
  const navigate = useNavigate();

  return (
    <Container
      className="d-flex flex-column align-items-center justify-content-center"
      style={{ minHeight: '100vh' }}
    >
      <div className="text-center">
        <h1 className="display-1 fw-bold text-primary">404</h1>
        <h2 className="mb-4">Page not found</h2>
        <p className="text-muted mb-4">
          The page you're looking for doesn't exist or has been moved.
        </p>
        <Button variant="primary" onClick={() => navigate('/')}>
          Go to Home
        </Button>
      </div>
    </Container>
  );
}
