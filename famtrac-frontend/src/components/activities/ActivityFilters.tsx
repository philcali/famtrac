import { Button } from '../common/Button';
import type { ActivityType } from '../../types/domain';

export interface ActivityFiltersProps {
  startDate: string;
  endDate: string;
  activityType: ActivityType | '';
  onStartDateChange: (date: string) => void;
  onEndDateChange: (date: string) => void;
  onActivityTypeChange: (type: ActivityType | '') => void;
  onClearFilters: () => void;
}

/**
 * ActivityFilters component for filtering activities by date range and type
 * - Provides date range filtering (Requirement 11.5)
 * - Provides activity type filtering (Requirement 11.6)
 */
export function ActivityFilters({
  startDate,
  endDate,
  activityType,
  onStartDateChange,
  onEndDateChange,
  onActivityTypeChange,
  onClearFilters,
}: ActivityFiltersProps) {
  const hasActiveFilters = startDate !== '' || endDate !== '' || activityType !== '';

  return (
    <div className="mb-4 p-3 border rounded-xl bg-gray-50">
      <h5 className="mb-3">Filter Activities</h5>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Start Date</label>
          <input
            type="date"
            value={startDate}
            onChange={(e) => onStartDateChange(e.target.value)}
            className="w-full px-3 py-2.5 rounded-xl border border-gray-200 bg-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30 focus:border-blue-400"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">End Date</label>
          <input
            type="date"
            value={endDate}
            onChange={(e) => onEndDateChange(e.target.value)}
            className="w-full px-3 py-2.5 rounded-xl border border-gray-200 bg-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30 focus:border-blue-400"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Activity Type</label>
          <select
            value={activityType}
            onChange={(e) => onActivityTypeChange(e.target.value as ActivityType | '')}
            className="w-full px-3 py-2.5 rounded-xl border border-gray-200 bg-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30 focus:border-blue-400"
          >
            <option value="">All Types</option>
            <option value="feeding">Feeding</option>
            <option value="diaper_change">Diaper Change</option>
            <option value="sleep">Sleep</option>
            <option value="pumping">Pumping</option>
            <option value="bath">Bath</option>
          </select>
        </div>
      </div>
      {hasActiveFilters && (
        <div className="text-end mt-3">
          <Button variant="secondary" size="sm" onClick={onClearFilters}>
            Clear Filters
          </Button>
        </div>
      )}
    </div>
  );
}
