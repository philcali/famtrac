import { useParams, useNavigate } from 'react-router-dom';
import { Container, Row, Col, Card } from 'react-bootstrap';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { Button } from '../components/common/Button';
import { formatAge } from '../utils/dateUtils';
import { useAuth } from '../auth/useAuth';
import { useApi } from '../hooks/useApi';
import { createApiClient } from '../api/client';
import { getDependent } from '../api/dependents';

/**
 * DependentDetailPage - View single dependent with details
 * - Displays dependent information with age (Requirements 7.2, 7.3, 7.4)
 * - Calculates and displays age based on date of birth (Requirement 7.4)
 * - Prepares for activity list integration (next phase)
 * - Provides navigation back to family (Requirement 14.3)
 */
export function DependentDetailPage() {
  const { dependentId } = useParams<{ dependentId: string }>();
  const navigate = useNavigate();
  const { getToken } = useAuth();

  const apiClient = createApiClient(getToken);

  // Fetch dependent details
  const {
    data: dependent,
    loading,
    error,
  } = useApi(() => getDependent(apiClient, dependentId ?? 'NA'), [dependentId]);

  const handleBackClick = () => {
    if (dependent) {
      navigate(`/families/${dependent.family_id}`);
    } else {
      navigate('/');
    }
  };

  if (loading) {
    return (
      <Container className="py-4">
        <LoadingSpinner />
      </Container>
    );
  }

  if (error) {
    return (
      <Container className="py-4">
        <ErrorMessage message={error} />
        <Button onClick={handleBackClick} className="mt-3">
          Back
        </Button>
      </Container>
    );
  }

  if (!dependent) {
    return (
      <Container className="py-4">
        <ErrorMessage message="Dependent not found" />
        <Button onClick={handleBackClick} className="mt-3">
          Back
        </Button>
      </Container>
    );
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  return (
    <Container className="py-4">
      <Row className="mb-4">
        <Col>
          <Button variant="secondary" onClick={handleBackClick} className="mb-3">
            ← Back to Family
          </Button>
          <h1>{dependent.name}</h1>
        </Col>
      </Row>

      {/* Dependent Information Card */}
      <Card className="mb-4">
        <Card.Body>
          <Card.Title>Dependent Information</Card.Title>
          <Card.Text>
            <strong>Age:</strong> {formatAge(dependent.date_of_birth)}
            <br />
            <strong>Date of Birth:</strong> {formatDate(dependent.date_of_birth)}
            <br />
            <br />
            <strong>Created:</strong> {formatDate(dependent.created_at)}
            <br />
            <strong>Updated:</strong> {formatDate(dependent.updated_at)}
          </Card.Text>
        </Card.Body>
      </Card>

      {/* Activities Section - Placeholder for next phase */}
      <Row className="mb-3">
        <Col>
          <h2>Activities</h2>
        </Col>
      </Row>

      <Card>
        <Card.Body>
          <Card.Text className="text-muted text-center py-4">
            Activity tracking will be available in the next phase.
          </Card.Text>
        </Card.Body>
      </Card>
    </Container>
  );
}
