import { Card } from 'react-bootstrap';
import { Button } from '../common/Button';
import { formatAge, formatDate } from '../../utils/dateUtils';
import type { Dependent } from '../../types/domain';

export interface DependentCardProps {
  dependent: Dependent;
  overrideTitle?: string;
  onEdit?: (dependent: Dependent) => void;
  onDelete?: (dependent: Dependent) => void;
  onView?: (dependent: Dependent) => void;
}

/**
 * DependentCard component displays a single dependent with action buttons
 * - Displays dependent name, age, and timestamps (Requirements 7.3, 7.4)
 * - Calculates and displays age based on date of birth (Requirement 7.4)
 * - Provides edit and delete actions (Requirements 8.1, 9.1)
 */
export function DependentCard({
  dependent,
  overrideTitle,
  onEdit,
  onDelete,
  onView,
}: DependentCardProps) {
  const options: Intl.DateTimeFormatOptions = {
    day: 'numeric',
    month: 'numeric',
    year: 'numeric',
  };
  return (
    <Card className="mb-3">
      <Card.Body>
        <Card.Title>{overrideTitle ?? dependent.name}</Card.Title>
        <Card.Text>
          <strong>Age:</strong> {formatAge(dependent.date_of_birth)}
          <br />
          <strong>Date of Birth:</strong> {formatDate(dependent.date_of_birth, options)}
        </Card.Text>
        <Card.Text className="text-muted small">
          Created: {formatDate(dependent.created_at, options)}
          <br />
          Updated: {formatDate(dependent.updated_at, options)}
        </Card.Text>
        {(onEdit || onDelete || onView) && (
          <div className="d-flex gap-2">
            {onView && (
              <Button variant="primary" size="sm" onClick={() => onView(dependent)}>
                View
              </Button>
            )}
            {onEdit && (
              <Button variant="secondary" size="sm" onClick={() => onEdit(dependent)}>
                Edit
              </Button>
            )}
            {onDelete && (
              <Button variant="danger" size="sm" onClick={() => onDelete(dependent)}>
                Delete
              </Button>
            )}
          </div>
        )}
      </Card.Body>
    </Card>
  );
}
