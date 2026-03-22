import { useState, useEffect } from 'react';
import { Form } from 'react-bootstrap';
import { Input } from '../common/Input';
import { Button } from '../common/Button';
import { useValidation } from '../../hooks/useValidation';
import { required, notFutureDate, positiveInteger } from '../../utils/validation';
import type { ActivityType, FeedingType, DiaperContents } from '../../types/domain';
import type { ActivityResponse } from '../../api/types';
import type { ValidationRule } from '../../hooks/useValidation';

export interface ActivityFormProps {
  activity?: ActivityResponse;
  familyId: string;
  dependentId: string;
  onSubmit: (data: {
    family_id: string;
    dependent_id: string;
    type: ActivityType;
    timestamp: string;
    feeding_type?: FeedingType;
    contents?: DiaperContents;
    start_time?: string;
    end_time?: string;
    volume_ml?: number;
  }) => Promise<void>;
  onCancel: () => void;
  loading?: boolean;
}

/**
 * ActivityForm component for creating and editing activities
 * - Validates activity type (Requirements 10.3)
 * - Validates timestamp not in future (Requirement 10.4)
 * - Conditional validation based on activity type:
 *   - Feeding: requires feeding_type (Requirement 10.5)
 *   - Diaper: requires contents (Requirement 10.6)
 *   - Sleep: requires start_time and end_time where end > start (Requirement 10.7)
 *   - Pumping: requires positive integer volume_ml (Requirement 10.8)
 * - Shows/hides fields based on selected activity type
 * - Displays validation errors (Requirement 18.2)
 * - Disables submit button during submission (Requirement 19.2)
 */
export function ActivityForm({
  activity,
  familyId,
  dependentId,
  onSubmit,
  onCancel,
  loading = false,
}: ActivityFormProps) {
  const formatISOValue = (date?: string | Date) => {
    const browserDate = new Date();
    let d: Date;
    if (date) {
      d = new Date(date);
    } else {
      d = new Date();
    }
    d.setTime(d.getTime() - browserDate.getTimezoneOffset() * 60 * 1000);
    return d.toISOString().slice(0, 16);
  };
  // Form state
  const [activityType, setActivityType] = useState<ActivityType>(activity?.type || 'feeding');
  const [timestamp, setTimestamp] = useState(() => {
    return formatISOValue(activity?.timestamp);
  });

  // Activity-specific fields
  const [feedingType, setFeedingType] = useState<FeedingType>(
    activity?.type === 'feeding' && activity.feeding_type ? activity.feeding_type : 'breast'
  );
  const [contents, setContents] = useState<DiaperContents>(
    activity?.type === 'diaper_change' && activity.contents ? activity.contents : 'wet'
  );
  const [startTime, setStartTime] = useState(() => {
    return formatISOValue(activity?.start_time);
  });
  const [endTime, setEndTime] = useState(() => {
    return formatISOValue(activity?.end_time);
  });
  const [volumeMl, setVolumeMl] = useState(
    (activity?.type === 'pumping' ||
      (activity?.type === 'feeding' && activity.feeding_type === 'bottle')) &&
      activity.volume_ml
      ? activity.volume_ml.toString()
      : ''
  );

  // Custom validation rule for sleep time range
  const sleepTimeRange = (): ValidationRule => {
    return () => {
      if (activityType === 'sleep') {
        const start = new Date(startTime);
        const end = new Date(endTime);

        if (isNaN(start.getTime()) || isNaN(end.getTime())) {
          return { isValid: false, error: 'Invalid time' };
        }

        if (end <= start) {
          return { isValid: false, error: 'End time must be after start time' };
        }
      }
      return { isValid: true, error: null };
    };
  };

  // Build validation rules dynamically based on activity type
  const getValidationRules = () => {
    const rules: Record<string, ValidationRule[]> = {
      timestamp: [required('Timestamp'), notFutureDate('Timestamp')],
    };

    if (activityType === 'feeding') {
      rules.feeding_type = [required('Feeding type')];
    } else if (activityType === 'diaper_change') {
      rules.contents = [required('Contents')];
    } else if (activityType === 'sleep') {
      rules.start_time = [required('Start time'), notFutureDate('Start time')];
      rules.end_time = [required('End time'), notFutureDate('End time'), sleepTimeRange()];
    } else if (activityType === 'pumping') {
      rules.volume_ml = [required('Volume'), positiveInteger('Volume')];
    }

    return rules;
  };

  const { validate, validateAll, errors, clearError, clearAllErrors } =
    useValidation(getValidationRules());

  // Clear errors when activity type changes
  useEffect(() => {
    clearAllErrors();
  }, [activityType, clearAllErrors]);

  const handleSubmit = async (e: React.SubmitEvent) => {
    e.preventDefault();

    // Build values object based on activity type
    const values: Record<string, unknown> = {
      timestamp,
    };

    if (activityType === 'feeding') {
      values.feeding_type = feedingType;
    } else if (activityType === 'diaper_change') {
      values.contents = contents;
    } else if (activityType === 'sleep') {
      values.start_time = startTime;
      values.end_time = endTime;
    } else if (activityType === 'pumping') {
      values.volume_ml = volumeMl;
    }

    const validation = validateAll(values);
    if (!validation.isValid) {
      return;
    }

    // Build submission data
    const data: {
      family_id: string;
      dependent_id: string;
      type: ActivityType;
      timestamp: string;
      feeding_type?: FeedingType;
      contents?: DiaperContents;
      start_time?: string;
      end_time?: string;
      volume_ml?: number;
    } = {
      family_id: familyId,
      dependent_id: dependentId,
      type: activityType,
      timestamp: new Date(timestamp).toISOString(),
    };

    if (activityType === 'feeding') {
      data.feeding_type = feedingType;
      if (feedingType === 'bottle') {
        data.volume_ml = parseInt(volumeMl, 10);
      }
    } else if (activityType === 'diaper_change') {
      data.contents = contents;
    } else if (activityType === 'sleep') {
      data.start_time = new Date(startTime).toISOString();
      data.end_time = new Date(endTime).toISOString();
    } else if (activityType === 'pumping') {
      data.volume_ml = parseInt(volumeMl, 10);
    }

    await onSubmit(data);
  };

  const handleTimestampBlur = () => {
    validate('timestamp', timestamp);
  };

  const handleTimestampChange = (value: string) => {
    setTimestamp(value);
    if (errors.timestamp) {
      clearError('timestamp');
    }
  };

  const handleStartTimeBlur = () => {
    validate('start_time', startTime);
  };

  const handleStartTimeChange = (value: string) => {
    setStartTime(value);
    if (errors.start_time) {
      clearError('start_time');
    }
  };

  const handleEndTimeBlur = () => {
    validate('end_time', endTime);
  };

  const handleEndTimeChange = (value: string) => {
    setEndTime(value);
    if (errors.end_time) {
      clearError('end_time');
    }
  };

  const handleVolumeMlBlur = () => {
    validate('volume_ml', volumeMl);
  };

  const handleVolumeMlChange = (value: string) => {
    setVolumeMl(value);
    if (errors.volume_ml) {
      clearError('volume_ml');
    }
  };

  const hasErrors = Object.keys(errors).length > 0;
  const isFormValid = !hasErrors && timestamp !== '';

  return (
    <Form onSubmit={handleSubmit}>
      <Form.Group className="mb-3">
        <Form.Label>
          Activity Type <span className="text-danger">*</span>
        </Form.Label>
        <Form.Select
          value={activityType}
          onChange={(e) => setActivityType(e.target.value as ActivityType)}
          disabled={loading || !!activity}
        >
          <option value="feeding">Feeding</option>
          <option value="diaper_change">Diaper Change</option>
          <option value="sleep">Sleep</option>
          <option value="pumping">Pumping</option>
        </Form.Select>
      </Form.Group>

      <Input
        label="Timestamp"
        type="datetime-local"
        value={timestamp}
        onChange={handleTimestampChange}
        onBlur={handleTimestampBlur}
        error={errors.timestamp}
        required
        disabled={loading}
      />

      {/* Feeding-specific fields */}
      {activityType === 'feeding' && (
        <Form.Group className="mb-3">
          <Form.Label>
            Feeding Type <span className="text-danger">*</span>
          </Form.Label>
          <Form.Select
            value={feedingType}
            onChange={(e) => setFeedingType(e.target.value as FeedingType)}
            disabled={loading}
            isInvalid={!!errors.feeding_type}
          >
            <option value="breast">Breast</option>
            <option value="bottle">Bottle</option>
            <option value="solid">Solid</option>
          </Form.Select>
          {errors.feeding_type && (
            <Form.Control.Feedback type="invalid">{errors.feeding_type}</Form.Control.Feedback>
          )}
        </Form.Group>
      )}

      {/* Diaper change-specific fields */}
      {activityType === 'diaper_change' && (
        <Form.Group className="mb-3">
          <Form.Label>
            Contents <span className="text-danger">*</span>
          </Form.Label>
          <Form.Select
            value={contents}
            onChange={(e) => setContents(e.target.value as DiaperContents)}
            disabled={loading}
            isInvalid={!!errors.contents}
          >
            <option value="wet">Wet</option>
            <option value="dirty">Dirty</option>
            <option value="both">Both</option>
          </Form.Select>
          {errors.contents && (
            <Form.Control.Feedback type="invalid">{errors.contents}</Form.Control.Feedback>
          )}
        </Form.Group>
      )}

      {/* Sleep-specific fields */}
      {activityType === 'sleep' && (
        <>
          <Input
            label="Start Time"
            type="datetime-local"
            value={startTime}
            onChange={handleStartTimeChange}
            onBlur={handleStartTimeBlur}
            error={errors.start_time}
            required
            disabled={loading}
          />
          <Input
            label="End Time"
            type="datetime-local"
            value={endTime}
            onChange={handleEndTimeChange}
            onBlur={handleEndTimeBlur}
            error={errors.end_time}
            required
            disabled={loading}
          />
        </>
      )}

      {/* Pumping-specific fields */}
      {(activityType === 'pumping' || feedingType === 'bottle') && (
        <Input
          label="Volume (ml)"
          type="number"
          value={volumeMl}
          onChange={handleVolumeMlChange}
          onBlur={handleVolumeMlBlur}
          error={errors.volume_ml}
          required
          placeholder="Enter volume in milliliters"
          disabled={loading}
        />
      )}

      <div className="d-flex gap-2">
        <Button
          type="submit"
          variant="primary"
          disabled={!isFormValid || loading}
          loading={loading}
        >
          {activity ? 'Update Activity' : 'Create Activity'}
        </Button>
        <Button type="button" variant="secondary" onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
      </div>
    </Form>
  );
}
