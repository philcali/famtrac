import { FamilyCard } from './FamilyCard';
import { LoadingSpinner } from '../common/LoadingSpinner';
import { ErrorMessage } from '../common/ErrorMessage';
import type { Family } from '../../types/domain';

export interface FamilyListProps {
  families: Family[];
  loading?: boolean;
  error?: string;
  onEdit: (family: Family) => void;
  onDelete: (family: Family) => void;
  onView: (family: Family) => void;
}

/**
 * FamilyList component displays all families for the user
 * - Displays loading indicator during fetch (Requirement 19.1)
 * - Displays error messages (Requirement 2.5)
 * - Renders all families (Requirement 3.1)
 */
export function FamilyList({
  families,
  loading,
  error,
  onEdit,
  onDelete,
  onView,
}: FamilyListProps) {
  if (loading) {
    return <LoadingSpinner />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  if (families.length === 0) {
    return (
      <div className="text-center text-muted py-5">
        <p>No families found. Create your first family to get started!</p>
      </div>
    );
  }

  return (
    <div>
      {families.map((family) => (
        <FamilyCard
          key={family.id}
          family={family}
          onEdit={onEdit}
          onDelete={onDelete}
          onView={onView}
        />
      ))}
    </div>
  );
}
