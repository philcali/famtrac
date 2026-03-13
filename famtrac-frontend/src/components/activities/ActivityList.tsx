import { useMemo } from 'react';
import { ActivityCard } from './ActivityCard';
import { SkeletonCard } from '../common/SkeletonCard';
import { ErrorMessage } from '../common/ErrorMessage';
import type { ActivityResponse } from '../../api/types';

export interface ActivityListProps {
  activities: ActivityResponse[];
  loading?: boolean;
  error?: string;
  onEdit: (activity: ActivityResponse) => void;
  onDelete: (activity: ActivityResponse) => void;
}

/**
 * ActivityList component displays all activities for a dependent
 * - Displays loading indicator during fetch (Requirement 19.1)
 * - Displays error messages (Requirement 10.11)
 * - Renders all activities in reverse chronological order (Requirement 11.3)
 * - Displays activity type, timestamp, and type-specific details (Requirements 11.4, 11.8)
 */
export function ActivityList({ activities, loading, error, onEdit, onDelete }: ActivityListProps) {
  // Sort activities in reverse chronological order (newest first)
  const sortedActivities = useMemo(() => {
    return [...activities].sort((a, b) => {
      const dateA = new Date(a.timestamp).getTime();
      const dateB = new Date(b.timestamp).getTime();
      return dateB - dateA; // Descending order (newest first)
    });
  }, [activities]);

  if (loading) {
    return <SkeletonCard count={3} />;
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  if (sortedActivities.length === 0) {
    return (
      <div className="text-center text-muted py-5">
        <p>No activities found. Add your first activity to get started!</p>
      </div>
    );
  }

  return (
    <div>
      {sortedActivities.map((activity) => (
        <ActivityCard key={activity.id} activity={activity} onEdit={onEdit} onDelete={onDelete} />
      ))}
    </div>
  );
}
