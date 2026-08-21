import { Badge } from '../common/Badge';
import { Button } from '../common/Button';
import { Icon } from '../common/Icon';
import type { ActivityResponse } from '../../api/types';
import { formatDateTime } from '../../utils/dateUtils';
import {
  getActivityTypeBadgeVariant,
  getActivityTypeLabel,
  renderActivityDetails,
} from './formats';

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
  const isStopwatchType = ['sleep', 'activity_time', 'tummy_time', 'wake_window'].includes(
    activity.type
  );
  const isInProgress = isStopwatchType && !activity.end_time;

  return (
    <div className="mb-3 p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
      <div className="flex justify-between items-start mb-2">
        <Badge variant={getActivityTypeBadgeVariant(activity.type)}>
          <Icon name={activity.type} size={14} className="me-1" />
          {getActivityTypeLabel(activity.type)}
        </Badge>
        <span className="text-muted text-sm">{formatDateTime(activity.timestamp)}</span>
      </div>
      <div className="text-sm mb-2">
        {renderActivityDetails(activity)}
        {isInProgress && <Icon name="stop" size={14} className="text-red-500 ms-1" />}
      </div>
      <div className="flex gap-2">
        <Button variant="secondary" size="sm" icon="pencil" onClick={() => onEdit(activity)}>
          Edit
        </Button>
        <Button variant="danger" size="sm" icon="trash" onClick={() => onDelete(activity)}>
          Delete
        </Button>
      </div>
    </div>
  );
}
