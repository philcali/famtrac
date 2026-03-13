import { Card } from 'react-bootstrap';

export interface SkeletonCardProps {
  count?: number;
}

/**
 * SkeletonCard component - Placeholder UI for loading cards
 * - Displays skeleton loading UI for data fetching (Requirement 19.3)
 */
export function SkeletonCard({ count = 1 }: SkeletonCardProps) {
  return (
    <>
      {Array.from({ length: count }).map((_, index) => (
        <Card key={index} className="mb-3">
          <Card.Body>
            <div className="skeleton skeleton-title mb-3"></div>
            <div className="skeleton skeleton-text mb-2"></div>
            <div className="skeleton skeleton-text mb-3" style={{ width: '80%' }}></div>
            <div className="d-flex gap-2">
              <div className="skeleton" style={{ width: '60px', height: '32px' }}></div>
              <div className="skeleton" style={{ width: '60px', height: '32px' }}></div>
              <div className="skeleton" style={{ width: '60px', height: '32px' }}></div>
            </div>
          </Card.Body>
        </Card>
      ))}
    </>
  );
}
