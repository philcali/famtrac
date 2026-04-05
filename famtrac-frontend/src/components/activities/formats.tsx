import { Spinner } from 'react-bootstrap';
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
    {!endTime && <Spinner animation="border" size="sm" />}
    <br />
    <strong>Duration:</strong>{' '}
    {startTime && endTime
      ? formatDuration(
          Math.round((new Date(endTime).getTime() - new Date(startTime).getTime()) / (1000 * 60))
        )
      : 'In Progress'}
  </>
);

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
          <strong>Volume:</strong> {activity.volume_ml ?? 'N/A'} ml
        </>
      );
    default:
      return null;
  }
};
