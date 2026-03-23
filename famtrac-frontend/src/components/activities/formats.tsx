import type { ActivityResponse } from '../../api/types';
import { formatDateTime, formatTime } from '../../utils/dateUtils';

export const formatActivityTimestamp = (activity: ActivityResponse) => {
  const now = new Date();
  const timestamp = new Date(activity.timestamp);

  if (now.toLocaleDateString() === timestamp.toLocaleDateString()) {
    return `Today at ${formatTime(timestamp)}`;
  } else {
    return formatDateTime(activity.timestamp);
  }
};

export const getActivityTypeLabel = (type: string) => {
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

export const getActivityTypeBadgeVariant = (type: string) => {
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

export const renderActivityDetails = (activity: ActivityResponse) => {
  switch (activity.type) {
    case 'feeding':
      return (
        <>
          <strong>Type:</strong>{' '}
          {activity.feeding_type
            ? activity.feeding_type.charAt(0).toUpperCase() + activity.feeding_type.slice(1)
            : 'N/A'}
          {activity.volume_ml && (
            <>
              <br />
              <strong>Volume:</strong>
              {' ' + activity.volume_ml}ml
            </>
          )}
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
                (new Date(activity.end_time).getTime() - new Date(activity.start_time).getTime()) /
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
