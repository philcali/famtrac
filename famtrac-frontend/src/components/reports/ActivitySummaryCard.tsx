import { Card, ListGroup } from 'react-bootstrap';

export interface ActivitySummaryCardProps {
  title: string;
  metrics: { label: string; value: string }[];
  variant: string;
}

/**
 * ActivitySummaryCard displays aggregated metrics for an activity type.
 * Renders a colored header and a list of label/value pairs.
 */
export function ActivitySummaryCard({ title, metrics, variant }: ActivitySummaryCardProps) {
  return (
    <Card className="mb-3">
      <Card.Header className={`bg-${variant} text-white`}>{title}</Card.Header>
      <ListGroup variant="flush">
        {metrics.map((metric) => (
          <ListGroup.Item key={metric.label} className="d-flex justify-content-between">
            <span>{metric.label}</span>
            <strong>{metric.value}</strong>
          </ListGroup.Item>
        ))}
      </ListGroup>
    </Card>
  );
}
