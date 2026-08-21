import { LoadingSpinner } from '../common/LoadingSpinner';
import type { ActivityResponse } from '../../api/types';
import { formatDateTime, formatTime } from '../../utils/dateUtils';
import { formatDuration } from '../../utils/formatDuration';

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
    case 'activity_time':
      return 'Activity Time';
    case 'tummy_time':
      return 'Tummy Time';
    case 'wake_window':
      return 'Wake Window';
    case 'bath':
      return 'Bath';
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
    case 'bath':
    case 'sleep':
      return 'info';
    case 'pumping':
      return 'success';
    case 'activity_time':
      return 'danger';
    case 'tummy_time':
      return 'dark';
    case 'wake_window':
      return 'light';
    default:
      return 'secondary';
  }
};

/** Renders the shared start / end / duration block for stopwatch-type activities. */
const renderTimedDetails = (startTime?: string, endTime?: string) => (
  <>
    <strong>Start:</strong> {startTime ? formatTime(startTime) : 'N/A'}
    <br />
    <strong>End: </strong>
    {endTime && <span>{formatTime(endTime)}</span>}
    {!endTime && <LoadingSpinner size="sm" />}
    <br />
    <strong>Duration:</strong>{' '}
    {startTime && endTime
      ? formatDuration(
          Math.round((new Date(endTime).getTime() - new Date(startTime).getTime()) / (1000 * 60))
        )
      : 'In Progress'}
  </>
);

export const isBottleExpired = (activity: ActivityResponse) => {
  if (activity.type !== 'feeding' || activity.feeding_type !== 'bottle') return false;
  const elapsed = Date.now() - new Date(activity.timestamp).getTime();
  return elapsed > 60 * 60 * 1000;
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
              {' ' + activity.volume_ml}ml / {Math.round((activity.volume_ml / 29.574) * 10) / 10}oz
            </>
          )}
          {activity.medicine_added && (
            <>
              <br />
              <strong>Medicine:</strong> Yes
            </>
          )}
          {isBottleExpired(activity) && (
            <>
              <br />
              <span className="text-red-500 font-semibold">Bottle expired</span>
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
    case 'wake_window':
      return renderTimedDetails(activity.start_time, activity.end_time);
    case 'activity_time':
      return (
        <>
          {renderTimedDetails(activity.start_time, activity.end_time)}
          {activity.description && (
            <>
              <br />
              <strong>Description:</strong> {activity.description}
            </>
          )}
        </>
      );
    case 'bath':
    case 'tummy_time':
      return (
        <>
          {renderTimedDetails(activity.start_time, activity.end_time)}
          {activity.notes && (
            <>
              <br />
              <strong>Notes:</strong> {activity.notes}
            </>
          )}
        </>
      );
    case 'pumping':
      return (
        <>
          <strong>Volume:</strong>{' '}
          {activity.volume_ml != null
            ? `${activity.volume_ml} ml / ${Math.round((activity.volume_ml / 29.574) * 10) / 10}oz`
            : 'N/A'}
        </>
      );
    default:
      return null;
  }
};
