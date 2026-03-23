import { ButtonGroup, Button, Form, Row, Col } from 'react-bootstrap';

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
    <Row className="mb-3 align-items-end g-3">
      <Col xs="auto">
        <ButtonGroup aria-label="Time range presets">
          {presets.map((preset) => (
            <Button
              key={preset.value}
              variant={activePreset === preset.value ? 'primary' : 'outline-primary'}
              onClick={() => onPresetSelect(preset.value)}
            >
              {preset.label}
            </Button>
          ))}
        </ButtonGroup>
      </Col>
      <Col xs="auto">
        <Form.Label htmlFor="report-start-date" className="visually-hidden">
          Start Date
        </Form.Label>
        <Form.Control
          id="report-start-date"
          type="date"
          value={startDate}
          onChange={(e) => onCustomRangeChange(e.target.value, endDate)}
          aria-label="Start date"
        />
      </Col>
      <Col xs="auto">
        <Form.Label htmlFor="report-end-date" className="visually-hidden">
          End Date
        </Form.Label>
        <Form.Control
          id="report-end-date"
          type="date"
          value={endDate}
          onChange={(e) => onCustomRangeChange(startDate, e.target.value)}
          aria-label="End date"
        />
      </Col>
    </Row>
  );
}
