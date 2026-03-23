import { useMemo } from 'react';
import { SkeletonCard } from '../common/SkeletonCard';
import { ErrorMessage } from '../common/ErrorMessage';
import { Button } from '../common/Button';
import type { ActivityResponse } from '../../api/types';
import { Badge, Table } from 'react-bootstrap';
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

  return (
    <>
      <Table responsive>
        <thead>
          <tr>
            <th>Type</th>
            <th>Start Time</th>
            <th>Description</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {sortedActivities.map((activity) => (
            <tr key={activity.id}>
              <td>
                <Badge bg={getActivityTypeBadgeVariant(activity.type)}>
                  {getActivityTypeLabel(activity.type)}
                </Badge>
              </td>
              <td>{formatActivityTimestamp(activity)}</td>
              <td>{renderActivityDetails(activity)}</td>
              <td>
                <Button
                  variant="secondary"
                  size="sm"
                  className="me-1 mb-1"
                  onClick={() => onEdit(activity)}
                >
                  Edit
                </Button>
                <Button
                  variant="danger"
                  size="sm"
                  className="mb-1"
                  onClick={() => onDelete(activity)}
                >
                  Delete
                </Button>
              </td>
            </tr>
          ))}
        </tbody>
      </Table>
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
