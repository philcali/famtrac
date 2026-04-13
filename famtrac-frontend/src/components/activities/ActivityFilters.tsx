import { Form, Row, Col, Button as BootstrapButton } from 'react-bootstrap';
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
    <div className="mb-4 p-3 border rounded bg-light">
      <h5 className="mb-3">Filter Activities</h5>
      <Row>
        <Col md={4}>
          <Form.Group className="mb-3">
            <Form.Label>Start Date</Form.Label>
            <Form.Control
              type="date"
              value={startDate}
              onChange={(e) => onStartDateChange(e.target.value)}
            />
          </Form.Group>
        </Col>
        <Col md={4}>
          <Form.Group className="mb-3">
            <Form.Label>End Date</Form.Label>
            <Form.Control
              type="date"
              value={endDate}
              onChange={(e) => onEndDateChange(e.target.value)}
            />
          </Form.Group>
        </Col>
        <Col md={4}>
          <Form.Group className="mb-3">
            <Form.Label>Activity Type</Form.Label>
            <Form.Select
              value={activityType}
              onChange={(e) => onActivityTypeChange(e.target.value as ActivityType | '')}
            >
              <option value="">All Types</option>
              <option value="feeding">Feeding</option>
              <option value="diaper_change">Diaper Change</option>
              <option value="sleep">Sleep</option>
              <option value="pumping">Pumping</option>
              <option value="bath">Bath</option>
            </Form.Select>
          </Form.Group>
        </Col>
      </Row>
      {hasActiveFilters && (
        <div className="text-end">
          <BootstrapButton variant="outline-secondary" size="sm" onClick={onClearFilters}>
            Clear Filters
          </BootstrapButton>
        </div>
      )}
    </div>
  );
}
