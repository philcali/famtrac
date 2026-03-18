import { Card, Badge } from 'react-bootstrap';
import { Button } from '../common/Button';
import type { ActivityResponse } from '../../api/types';
import { formatDateTime, formatTime } from '../../utils/dateUtils';

export interface ActivityCardProps {
  activity: ActivityResponse;
  onEdit: (activity: ActivityResponse) => void;
  onDelete: (activity: ActivityResponse) => void;
}

/**
 * ActivityCard component displays a single activity with action buttons
 * - Displays activity type, timestamp, and type-specific details (Requirements 11.4, 11.8)
 * - Provides edit and delete actions (Requirements 12.1, 13.1)
 */
export function ActivityCard({ activity, onEdit, onDelete }: ActivityCardProps) {
  const getActivityTypeLabel = (type: string) => {
    switch (type) {
      case 'feeding':
        return 'Feeding';
      case 'diaper_change':
        return 'Diaper Change';
      case 'sleep':
        return 'Sleep';
      case 'pumping':
        return 'Pumping';
      default:
        return type;
    }
  };

  const getActivityTypeBadgeVariant = (type: string) => {
    switch (type) {
      case 'feeding':
        return 'primary';
      case 'diaper_change':
        return 'warning';
      case 'sleep':
        return 'info';
      case 'pumping':
        return 'success';
      default:
        return 'secondary';
    }
  };

  const renderActivityDetails = () => {
    switch (activity.type) {
      case 'feeding':
        return (
          <>
            <strong>Type:</strong>{' '}
            {activity.feeding_type
              ? activity.feeding_type.charAt(0).toUpperCase() + activity.feeding_type.slice(1)
              : 'N/A'}
          </>
        );
      case 'diaper_change':
        return (
          <>
            <strong>Contents:</strong>{' '}
            {activity.contents
              ? activity.contents.charAt(0).toUpperCase() + activity.contents.slice(1)
              : 'N/A'}
          </>
        );
      case 'sleep':
        return (
          <>
            <strong>Start:</strong> {activity.start_time ? formatTime(activity.start_time) : 'N/A'}
            <br />
            <strong>End:</strong> {activity.end_time ? formatTime(activity.end_time) : 'N/A'}
            <br />
            <strong>Duration:</strong>{' '}
            {activity.start_time && activity.end_time
              ? Math.round(
                  (new Date(activity.end_time).getTime() -
                    new Date(activity.start_time).getTime()) /
                    (1000 * 60)
                )
              : 0}{' '}
            minutes
          </>
        );
      case 'pumping':
        return (
          <>
            <strong>Volume:</strong> {activity.volume_ml ?? 'N/A'} ml
          </>
        );
      default:
        return null;
    }
  };

  return (
    <Card className="mb-3">
      <Card.Body>
        <div className="d-flex justify-content-between align-items-start mb-2">
          <Badge bg={getActivityTypeBadgeVariant(activity.type)}>
            {getActivityTypeLabel(activity.type)}
          </Badge>
          <span className="text-muted small">{formatDateTime(activity.timestamp)}</span>
        </div>
        <Card.Text>{renderActivityDetails()}</Card.Text>
        <div className="d-flex gap-2">
          <Button variant="secondary" size="sm" onClick={() => onEdit(activity)}>
            Edit
          </Button>
          <Button variant="danger" size="sm" onClick={() => onDelete(activity)}>
            Delete
          </Button>
        </div>
      </Card.Body>
    </Card>
  );
}
