import { useMemo } from 'react';
import { SkeletonCard } from '../common/SkeletonCard';
import { ErrorMessage } from '../common/ErrorMessage';
import { Button } from '../common/Button';
import type { ActivityResponse } from '../../api/types';
import { Badge } from '../common/Badge';
import {
  formatActivityTimestamp,
  getActivityTypeBadgeVariant,
  getActivityTypeLabel,
  renderActivityDetails,
} from './formats';

export interface ActivityListProps {
  activities: ActivityResponse[];
  loading?: boolean;
  error?: string;
  hasMore?: boolean;
  loadingMore?: boolean;
  onLoadMore?: () => void;
  onEdit: (activity: ActivityResponse) => void;
  onDelete: (activity: ActivityResponse) => void;
  onStop: (activity: ActivityResponse) => void;
}

/**
 * ActivityList component displays all activities for a dependent
 * - Displays loading indicator during fetch (Requirement 19.1)
 * - Displays error messages (Requirement 10.11)
 * - Renders all activities in reverse chronological order (Requirement 11.3)
 * - Displays activity type, timestamp, and type-specific details (Requirements 11.4, 11.8)
 * - Shows "Load More" button when hasMore is true
 * - Calls onLoadMore when clicked
 * - Disables "Load More" and shows spinner when loadingMore
 */
export function ActivityList({
  activities,
  loading,
  error,
  hasMore,
  loadingMore,
  onLoadMore,
  onEdit,
  onDelete,
  onStop,
}: ActivityListProps) {
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

  const isProgressing = (activity: ActivityResponse) => {
    const isStopwatchType = ['sleep', 'activity_time', 'tummy_time', 'wake_window'].includes(
      activity.type
    );
    return isStopwatchType && !activity.end_time;
  };

  return (
    <>
      <div className="overflow-x-auto">
        <table className="w-full text-sm text-left">
          <thead>
            <tr className="border-b border-gray-200">
              <th className="py-2 pr-4 font-medium text-gray-500">Type</th>
              <th className="py-2 pr-4 font-medium text-gray-500">Start Time</th>
              <th className="py-2 pr-4 font-medium text-gray-500">Description</th>
              <th className="py-2 font-medium text-gray-500">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {sortedActivities.map((activity) => (
              <tr key={activity.id} className="align-top">
                <td className="py-3 pr-4 align-top">
                  <Badge variant={getActivityTypeBadgeVariant(activity.type)}>
                    {getActivityTypeLabel(activity.type)}
                  </Badge>
                </td>
                <td className="py-3 pr-4 align-top text-sm text-gray-500">
                  {formatActivityTimestamp(activity)}
                </td>
                <td className="py-3 pr-4 align-top text-sm">{renderActivityDetails(activity)}</td>
                <td className="py-3 align-top">
                  <div className="flex gap-1">
                    <Button
                      variant="secondary"
                      size="sm"
                      icon="pencil"
                      onClick={() => onEdit(activity)}
                    ></Button>
                    {isProgressing(activity) && (
                      <Button
                        variant="secondary"
                        size="sm"
                        icon="stop"
                        onClick={() => onStop(activity)}
                      ></Button>
                    )}
                    <Button
                      variant="danger"
                      size="sm"
                      icon="trash"
                      onClick={() => onDelete(activity)}
                    ></Button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {hasMore && (
        <div className="text-center mt-3">
          <Button
            variant="secondary"
            onClick={onLoadMore}
            disabled={loadingMore}
            loading={loadingMore}
          >
            Load More
          </Button>
        </div>
      )}
    </>
  );
}
