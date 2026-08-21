import { useState, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ErrorMessage } from '../components/common/ErrorMessage';
import { LoadingSpinner } from '../components/common/LoadingSpinner';
import { Button } from '../components/common/Button';
import { TimeRangeSelector } from '../components/reports/TimeRangeSelector';
import { ActivitySummaryCard } from '../components/reports/ActivitySummaryCard';
import { ActivityChart } from '../components/reports/ActivityChart';
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
  computeWakeWindowSummary,
  computeBathSummary,
  transformSleepChartData,
  transformWakeWindowChartData,
  transformBathChartData,
  transformDiaperStackedChartData,
  transformVolumeTrendData,
  transformVolumeCompositeData,
} from '../utils/reportUtils';
import type { TimeRangePreset, TrendWindow } from '../utils/reportUtils';
import { formatDuration } from '../utils/formatDuration';

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
  const [trendWindow, setTrendWindow] = useState<TrendWindow>('1h');

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
  const wakeWindowSummary = useMemo(() => computeWakeWindowSummary(activities), [activities]);
  const bathSummary = useMemo(() => computeBathSummary(activities), [activities]);

  // Compute chart data from activities
  const feedingCompositeData = useMemo(
    () => transformVolumeCompositeData(activities, 'feeding', trendWindow),
    [activities, trendWindow]
  );
  const sleepChartData = useMemo(() => transformSleepChartData(activities), [activities]);
  const wakeWindowChartData = useMemo(() => transformWakeWindowChartData(activities), [activities]);
  const bathChartData = useMemo(() => transformBathChartData(activities), [activities]);
  const diaperChartData = useMemo(() => transformDiaperStackedChartData(activities), [activities]);
  const pumpingChartData = useMemo(
    () => transformVolumeTrendData(activities, 'pumping', trendWindow),
    [activities, trendWindow]
  );

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

  // Loading state
  if (dependentLoading) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <LoadingSpinner />
      </div>
    );
  }

  // Error state for dependent fetch
  if (dependentError) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <ErrorMessage message={dependentError} />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Dependent
        </Button>
      </div>
    );
  }

  if (!dependent) {
    return (
      <div className="py-4 max-w-5xl mx-auto px-4">
        <ErrorMessage message="Dependent not found" />
        <Button onClick={handleBackClick} className="mt-3">
          ← Back to Dependent
        </Button>
      </div>
    );
  }

  return (
    <div className="py-4 max-w-5xl mx-auto px-4">
      {/* Page heading with dependent name and back navigation */}
      <div className="mb-4">
        <h2 className="heading">
          Reports
          <Button variant="secondary" onClick={handleBackClick} className="heading-right">
            ← Back to {dependent.name}
          </Button>
        </h2>
      </div>

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
        <div className="mt-4 p-4 bg-blue-50 border border-blue-100 rounded-xl text-blue-700 text-sm">
          No activities found for the selected time period.
        </div>
      )}

      {/* Summary cards */}
      {!activitiesLoading && !activitiesError && activities.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <ActivitySummaryCard
            title="Feeding"
            variant="success"
            metrics={[
              { label: 'Total Feedings', value: String(feedingSummary.totalCount) },
              { label: 'Total Volume', value: `${Math.round(feedingSummary.totalVolumeMl)} ml` },
              { label: 'Avg Volume', value: `${Math.round(feedingSummary.averageVolumeMl)} ml` },
              { label: 'With Medicine', value: String(feedingSummary.medicineCount) },
            ]}
          />
          <ActivitySummaryCard
            title="Sleep"
            variant="info"
            metrics={[
              { label: 'Total Sessions', value: String(sleepSummary.totalCount) },
              {
                label: 'Total Duration',
                value: formatDuration(sleepSummary.totalDurationMinutes),
              },
              {
                label: 'Avg Duration',
                value: formatDuration(sleepSummary.averageDurationMinutes),
              },
            ]}
          />
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
          <ActivitySummaryCard
            title="Pumping"
            variant="primary"
            metrics={[
              { label: 'Total Sessions', value: String(pumpingSummary.totalCount) },
              { label: 'Total Volume', value: `${Math.round(pumpingSummary.totalVolumeMl)} ml` },
              { label: 'Avg Volume', value: `${Math.round(pumpingSummary.averageVolumeMl)} ml` },
            ]}
          />
          <ActivitySummaryCard
            title="Wake Windows"
            variant="secondary"
            metrics={[
              { label: 'Total Sessions', value: String(wakeWindowSummary.totalCount) },
              {
                label: 'Total Duration',
                value: formatDuration(wakeWindowSummary.totalDurationMinutes),
              },
              {
                label: 'Avg Duration',
                value: formatDuration(wakeWindowSummary.averageDurationMinutes),
              },
            ]}
          />
          <ActivitySummaryCard
            title="Baths"
            variant="info"
            metrics={[
              { label: 'Total Baths', value: String(bathSummary.totalCount) },
              {
                label: 'Total Duration',
                value: formatDuration(bathSummary.totalDurationMinutes),
              },
              {
                label: 'Avg Duration',
                value: formatDuration(bathSummary.averageDurationMinutes),
              },
            ]}
          />
        </div>
      )}

      {/* Charts */}
      {!activitiesLoading && !activitiesError && activities.length > 0 && (
        <>
          <div className="mt-4 mb-3 flex items-center gap-2">
            <span className="text-muted">Trend window:</span>
            <div className="inline-flex rounded-lg border border-gray-200 overflow-hidden">
              {(['1h', '6h', '1d'] as TrendWindow[]).map((w) => (
                <Button
                  key={w}
                  variant={trendWindow === w ? 'primary' : 'outline-primary'}
                  onClick={() => setTrendWindow(w)}
                  className="rounded-none px-3 py-1.5 text-sm"
                >
                  {w === '1h' ? '1 Hour' : w === '6h' ? '6 Hours' : '1 Day'}
                </Button>
              ))}
            </div>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <ActivityChart
              title="Feeding Volume Trend"
              data={feedingCompositeData}
              chartType="composite"
              yAxisLabel="Volume (ml)"
              xAxisLabel="Time"
              compositeBarColor="#28a745"
              compositeLineColor="#155724"
              emptyMessage="No feeding data with volume for the selected period."
            />
            <ActivityChart
              title="Sleep Duration Per Day"
              data={sleepChartData}
              chartType="bar"
              yAxisLabel="Duration (hours)"
              xAxisLabel="Day"
              color="#17a2b8"
              emptyMessage="No sleep data for the selected period."
            />
            <ActivityChart
              title="Wake Window Duration Per Day"
              data={wakeWindowChartData}
              chartType="bar"
              yAxisLabel="Duration (hours)"
              xAxisLabel="Day"
              color="#fd7e14"
              emptyMessage="No wake window data for the selected period."
            />
            <ActivityChart
              title="Diaper Changes Per Day"
              data={diaperChartData}
              chartType="stacked-bar"
              yAxisLabel="Count"
              xAxisLabel="Day"
              stackedBars={[
                { dataKey: 'wet', color: '#17a2b8', name: 'Wet' },
                { dataKey: 'dirty', color: '#6f4e37', name: 'Dirty' },
                { dataKey: 'both', color: '#ffc107', name: 'Both' },
              ]}
              emptyMessage="No diaper change data for the selected period."
            />
            <ActivityChart
              title="Pumping Volume Trend (avg)"
              data={pumpingChartData}
              chartType="line"
              yAxisLabel="Avg Volume (ml)"
              xAxisLabel="Time"
              color="#007bff"
              emptyMessage="No pumping data with volume for the selected period."
            />
            <ActivityChart
              title="Baths Per Day"
              data={bathChartData}
              chartType="bar"
              yAxisLabel="Count"
              xAxisLabel="Day"
              color="#0dcaf0"
              emptyMessage="No bath data for the selected period."
            />
          </div>
        </>
      )}
    </div>
  );
}
