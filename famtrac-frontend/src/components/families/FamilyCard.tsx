import { Card } from 'react-bootstrap';
import { Button } from '../common/Button';
import type { Family } from '../../types/domain';
import { formatDate } from '../../utils/dateUtils';

export interface FamilyCardProps {
  family: Family;
  onEdit: (family: Family) => void;
  onDelete: (family: Family) => void;
  onView: (family: Family) => void;
}

/**
 * FamilyCard component displays a single family with action buttons
 * - Displays family name and timestamps (Requirement 3.3, 3.4)
 * - Provides edit and delete actions (Requirements 4.1, 5.1)
 */
export function FamilyCard({ family, onEdit, onDelete, onView }: FamilyCardProps) {
  return (
    <Card className="mb-3">
      <Card.Body>
        <Card.Title>{family.name}</Card.Title>
        <Card.Text className="text-muted small">
          Created: {formatDate(family.created_at)}
          <br />
          Updated: {formatDate(family.updated_at)}
        </Card.Text>
        <div className="d-flex gap-2">
          <Button variant="primary" size="sm" onClick={() => onView(family)}>
            View
          </Button>
          <Button variant="secondary" size="sm" onClick={() => onEdit(family)}>
            Edit
          </Button>
          <Button variant="danger" size="sm" onClick={() => onDelete(family)}>
            Delete
          </Button>
        </div>
      </Card.Body>
    </Card>
  );
}
