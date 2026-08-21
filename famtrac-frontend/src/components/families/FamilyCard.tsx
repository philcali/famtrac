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
    <div className="mb-3 p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
      <h3 className="text-base font-semibold mb-2">{family.name}</h3>
      <div className="text-sm text-muted">
        Created: {formatDate(family.created_at)}
        <br />
        Updated: {formatDate(family.updated_at)}
      </div>
      <div className="flex gap-2 mt-3">
        <Button variant="primary" size="sm" icon="eye" onClick={() => onView(family)}></Button>
        <Button
          variant="secondary"
          size="sm"
          icon="pencil"
          onClick={() => onEdit(family)}
        ></Button>
        <Button variant="danger" size="sm" icon="trash" onClick={() => onDelete(family)}></Button>
      </div>
    </div>
  );
}
