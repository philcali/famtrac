import { useState, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Row, Col, Alert } from 'react-bootstrap';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { Button } from '../components/common/Button';
import { TimeRangeSelector } from '../components/reports/TimeRangeSelector';
import { ActivitySummaryCard } from '../components/reports/ActivitySummaryCard';
import { useAuth } from '../auth/useAuth';
import { useApi } from '../hooks/useApi';
import { useReportData } from '../hooks/useReportData';
import { createApiClient } from '../api/client';
import { getDependent } from '../api/dependents';
import {
  getPresetDateRange,
  computeFeedingSummary,
  computeSleepSummary,
  computeDiaperSummary,
  computePumpingSummary,
} from '../utils/reportUtils';
import type { TimeRangePreset } from '../utils/reportUtils';

/**
 * ReportPage - Displays activity summaries for a dependent over a configurable time range.
 * - Reads familyId and dependentId from route params
 * - Fetches dependent details for page heading (Requirement 1.2)
 * - Manages time range state, defaulting to "Today" preset (Requirement 2.9)
 * - Uses useReportData hook to fetch all activities (Requirements 3.1–3.3)
 * - Computes summaries for feeding, sleep, diaper, pumping (Requirements 4.1–4.3, 5.1–5.3, 6.1–6.2, 7.1–7.3)
 * - Shows loading spinner, error message, and global empty state (Requirements 3.2, 3.3, 9.1)
 * - Provides "Back to Dependent" navigation link (Requirement 1.3)
 */
export function ReportPage() {
  const { familyId, dependentId } = useParams<{ familyId: string; dependentId: string }>();
  const navigate = useNavigate();
  const { getToken } = useAuth();
  const apiClient = createApiClient(getToken);

  // Default to "Today" preset
  const defaultRange = getPresetDateRange('today');
  const [startDate, setStartDate] = useState(defaultRange.startDate);
  const [endDate, setEndDate] = useState(defaultRange.endDate);
  const [activePreset, setActivePreset] = useState<TimeRangePreset | null>('today');

  // Fetch dependent details for page heading
  const {
    data: dependent,
    loading: dependentLoading,
    error: dependentError,
  } = useApi(
    () => getDependent(apiClient, familyId ?? 'NA', dependentId ?? 'NA'),
    [familyId, dependentId]
  );

  // Fetch all activities for the selected date range
  const {
    activities,
    loading: activitiesLoading,
    error: activitiesError,
  } = useReportData(familyId ?? 'NA', dependentId ?? 'NA', startDate, endDate);

  // Compute summaries from activities
  const feedingSummary = useMemo(() => computeFeedingSummary(activities), [activities]);
  const sleepSummary = useMemo(() => computeSleepSummary(activities), [activities]);
  const diaperSummary = useMemo(() => computeDiaperSummary(activities), [activities]);
  const pumpingSummary = useMemo(() => computePumpingSummary(activities), [activities]);

  // Handlers
  const handleBackClick = () => {
    if (familyId && dependentId) {
      navigate(`/families/${familyId}/dependents/${dependentId}`);
    } else if (familyId) {
      navigate(`/families/${familyId}`);
    } else {
      navigate('/');
    }
  };

  const handlePresetSelect = (preset: TimeRangePreset) => {
    const range = getPresetDateRange(preset);
    setStartDate(range.startDate);
    setEndDate(range.endDate);
    setActivePreset(preset);
  };

  const handleCustomRangeChange = (newStartDate: string, newEndDate: string) => {
    setStartDate(newStartDate);
    setEndDate(newEndDate);
    setActivePreset(null);
  };

  // Format helpers for display
  const formatMinutes = (minutes: number): string => {
    const hours = Math.floor(minutes / 60);
    const mins = Math.round(minutes % 60);
    if (hours > 0) {
      return `${hours}h ${mins}m`;
    }
    return `${mins}m`;
  };

  // Loading state
  if (dependentLoading) {
    return (
      <Container className="py-4">
        <LoadingSpinner />
      </Container>
    );
  }

  // Error state for dependent fetch
  if (dependentError) {
    return (
      <Container className="py-4">
        <ErrorMessage message={dependentError} />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Dependent
        </Button>
      </Container>
    );
  }

  if (!dependent) {
    return (
      <Container className="py-4">
        <ErrorMessage message="Dependent not found" />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Dependent
        </Button>
      </Container>
    );
  }

  return (
    <Container className="py-4">
      {/* Page heading with dependent name and back navigation */}
      <Row className="mb-4">
        <Col>
          <h2 className="heading">
            Reports for {dependent.name}
            <Button variant="secondary" onClick={handleBackClick} className="heading-right">
              ← Back to Dependent
            </Button>
          </h2>
        </Col>
      </Row>

      {/* Time range selector */}
      <TimeRangeSelector
        startDate={startDate}
        endDate={endDate}
        activePreset={activePreset}
        onPresetSelect={handlePresetSelect}
        onCustomRangeChange={handleCustomRangeChange}
      />

      {/* Activities loading state */}
      {activitiesLoading && <LoadingSpinner text="Loading activities..." />}

      {/* Activities error state */}
      {activitiesError && <ErrorMessage message={activitiesError} />}

      {/* Global empty state */}
      {!activitiesLoading && !activitiesError && activities.length === 0 && (
        <Alert variant="info">No activities found for the selected time period.</Alert>
      )}

      {/* Summary cards */}
      {!activitiesLoading && !activitiesError && activities.length > 0 && (
        <Row>
          <Col md={6} lg={3}>
            <ActivitySummaryCard
              title="Feeding"
              variant="success"
              metrics={[
                { label: 'Total Feedings', value: String(feedingSummary.totalCount) },
                { label: 'Total Volume', value: `${Math.round(feedingSummary.totalVolumeMl)} ml` },
                { label: 'Avg Volume', value: `${Math.round(feedingSummary.averageVolumeMl)} ml` },
              ]}
            />
          </Col>
          <Col md={6} lg={3}>
            <ActivitySummaryCard
              title="Sleep"
              variant="info"
              metrics={[
                { label: 'Total Sessions', value: String(sleepSummary.totalCount) },
                {
                  label: 'Total Duration',
                  value: formatMinutes(sleepSummary.totalDurationMinutes),
                },
                {
                  label: 'Avg Duration',
                  value: formatMinutes(sleepSummary.averageDurationMinutes),
                },
              ]}
            />
          </Col>
          <Col md={6} lg={3}>
            <ActivitySummaryCard
              title="Diaper Changes"
              variant="warning"
              metrics={[
                { label: 'Total Changes', value: String(diaperSummary.totalCount) },
                { label: 'Wet', value: String(diaperSummary.wetCount) },
                { label: 'Dirty', value: String(diaperSummary.dirtyCount) },
                { label: 'Both', value: String(diaperSummary.bothCount) },
              ]}
            />
          </Col>
          <Col md={6} lg={3}>
            <ActivitySummaryCard
              title="Pumping"
              variant="primary"
              metrics={[
                { label: 'Total Sessions', value: String(pumpingSummary.totalCount) },
                { label: 'Total Volume', value: `${Math.round(pumpingSummary.totalVolumeMl)} ml` },
                { label: 'Avg Volume', value: `${Math.round(pumpingSummary.averageVolumeMl)} ml` },
              ]}
            />
          </Col>
        </Row>
      )}
    </Container>
  );
}
