import { DependentCard } from './DependentCard';
import { LoadingSpinner } from '../common/LoadingSpinner';
import { ErrorMessage } from '../common/ErrorMessage';
import type { Dependent } from '../../types/domain';

export interface DependentListProps {
  dependents: Dependent[];
  loading?: boolean;
  error?: string;
  onEdit: (dependent: Dependent) => void;
  onDelete: (dependent: Dependent) => void;
  onView: (dependent: Dependent) => void;
}

/**
 * DependentList component displays all dependents for a family
 * - Displays loading indicator during fetch (Requirement 19.1)
 * - Displays error messages (Requirement 6.7)
 * - Renders all dependents (Requirement 7.1)
 */
export function DependentList({
  dependents,
  loading,
  error,
  onEdit,
  onDelete,
  onView,
}: DependentListProps) {
  if (loading) {
    return <LoadingSpinner />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  if (dependents.length === 0) {
    return (
      <div className="text-center text-muted py-5">
        <p>No dependents found. Add your first dependent to get started!</p>
      </div>
    );
  }

  return (
    <div>
      {dependents.map((dependent) => (
        <DependentCard
          key={dependent.id}
          dependent={dependent}
          onEdit={onEdit}
          onDelete={onDelete}
          onView={onView}
        />
      ))}
    </div>
  );
}
