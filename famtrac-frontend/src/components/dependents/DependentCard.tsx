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
    <div className="mb-3 p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
      <h3 className="text-base font-semibold mb-2">{overrideTitle ?? dependent.name}</h3>
      <div className="text-sm mb-2">
        <strong>Age:</strong> {formatAge(dependent.date_of_birth)}
        <br />
        <strong>Date of Birth:</strong> {formatDate(dependent.date_of_birth, options)}
      </div>
      <div className="text-sm text-muted">
        Created: {formatDate(dependent.created_at, options)}
        <br />
        Updated: {formatDate(dependent.updated_at, options)}
      </div>
      {(onEdit || onDelete || onView) && (
        <div className="flex gap-2 mt-3">
          {onView && (
            <Button
              variant="primary"
              size="sm"
              icon="eye"
              onClick={() => onView(dependent)}
            ></Button>
          )}
          {onEdit && (
            <Button
              variant="secondary"
              size="sm"
              icon="pencil"
              onClick={() => onEdit(dependent)}
            ></Button>
          )}
          {onDelete && (
            <Button
              variant="danger"
              size="sm"
              icon="trash"
              onClick={() => onDelete(dependent)}
            ></Button>
          )}
        </div>
      )}
    </div>
  );
}
