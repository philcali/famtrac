import { Button } from '../common/Button';

type TimeRangePreset = 'today' | 'week' | 'month';

interface TimeRangeSelectorProps {
  startDate: string;
  endDate: string;
  activePreset: TimeRangePreset | null;
  onPresetSelect: (preset: TimeRangePreset) => void;
  onCustomRangeChange: (startDate: string, endDate: string) => void;
}

const presets: { value: TimeRangePreset; label: string }[] = [
  { value: 'today', label: 'Today' },
  { value: 'week', label: 'This Week' },
  { value: 'month', label: 'This Month' },
];

/**
 * TimeRangeSelector - Preset buttons and custom date inputs for report filtering
 * - Displays three preset quick-link buttons (Requirement 2.1)
 * - Custom start/end date inputs (Requirement 2.5)
 * - Highlights active preset (Requirement 2.7)
 * - Deselects preset on custom range change (Requirement 2.8)
 */
export function TimeRangeSelector({
  startDate,
  endDate,
  activePreset,
  onPresetSelect,
  onCustomRangeChange,
}: TimeRangeSelectorProps) {
  return (
    <div className="mb-3 flex flex-col sm:flex-row items-start sm:items-end gap-3">
      <div className="flex gap-1" aria-label="Time range presets">
        {presets.map((preset) => (
          <Button
            key={preset.value}
            variant={activePreset === preset.value ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => onPresetSelect(preset.value)}
          >
            {preset.label}
          </Button>
        ))}
      </div>
      <div className="flex flex-col sm:flex-row gap-2">
        <div>
          <label
            htmlFor="report-start-date"
            className="block text-xs font-medium text-gray-500 mb-1"
          >
            Start Date
          </label>
          <input
            id="report-start-date"
            type="date"
            value={startDate}
            onChange={(e) => onCustomRangeChange(e.target.value, endDate)}
            className="px-3 py-2 rounded-xl border border-gray-200 bg-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30 focus:border-blue-400"
          />
        </div>
        <div>
          <label htmlFor="report-end-date" className="block text-xs font-medium text-gray-500 mb-1">
            End Date
          </label>
          <input
            id="report-end-date"
            type="date"
            value={endDate}
            onChange={(e) => onCustomRangeChange(startDate, e.target.value)}
            className="px-3 py-2 rounded-xl border border-gray-200 bg-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30 focus:border-blue-400"
          />
        </div>
      </div>
    </div>
  );
}
