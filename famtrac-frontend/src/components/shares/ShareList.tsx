import { Row, Col, Table } from 'react-bootstrap';
import { SkeletonCard } from '../common/SkeletonCard';
import { ErrorMessage } from '../common/ErrorMessage';
import { Button } from '../common/Button';
import type { Share } from '../../types/domain';
import { ShareStatusBadge } from './ShareStatusBadge';
import { SharePermissionBadges } from './SharePermissionBadges';

export interface ShareListProps {
  shares: Share[];
  loading?: boolean;
  error?: string;
  hasMore?: boolean;
  loadingMore?: boolean;
  onLoadMore?: () => void;
  onEdit: (share: Share) => void;
  onRevoke: (share: Share) => void;
}

/**
 * ShareList renders ShareCards in a grid with loading, error, empty, and pagination states.
 * Follows the DependentList pattern.
 * - Renders ShareCard for each share (Requirement 6.1)
 * - Shows SkeletonCard when loading (Requirement 6.2)
 * - Shows empty state message (Requirement 6.3)
 * - Shows ErrorMessage on error (Requirement 6.4)
 * - Shows "Load More" button when hasMore is true (Requirement 6.5)
 * - Calls onLoadMore when clicked (Requirement 6.6)
 * - Disables "Load More" and shows spinner when loadingMore (Requirement 6.7)
 */
export function ShareList({
  shares,
  loading,
  error,
  hasMore,
  loadingMore,
  onLoadMore,
  onEdit,
  onRevoke,
}: ShareListProps) {
  if (loading) {
    return (
      <Row>
        <Col xs={12} md={6} lg={4}>
          <SkeletonCard count={3} />
        </Col>
      </Row>
    );
  }

  if (error) {
    return <ErrorMessage message={error} />;
  }

  if (shares.length === 0) {
    return (
      <div className="text-center text-muted py-5">
        <p>No shares found. Invite a user to share this family!</p>
      </div>
    );
  }

  return (
    <>
      <Table responsive>
        <thead>
          <tr>
            <th>Username</th>
            <th>Status</th>
            <th>Permissions</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {shares.map((share) => (
            <tr key={share.id}>
              <td>{share.accepter_username}</td>
              <td>
                <ShareStatusBadge status={share.status} />
              </td>
              <td>
                <SharePermissionBadges share={share} />
              </td>
              <td>
                <Button
                  variant="secondary"
                  size="sm"
                  className="mb-1"
                  onClick={() => onEdit(share)}
                  disabled={share.status === 'expired'}
                >
                  Edit
                </Button>{' '}
                <Button variant="danger" size="sm" className="mb-1" onClick={() => onRevoke(share)}>
                  Revoke
                </Button>
              </td>
            </tr>
          ))}
        </tbody>
      </Table>
      {hasMore && (
        <div className="text-center mt-3">
          <Button
            variant="secondary"
            onClick={onLoadMore}
            disabled={loadingMore}
            loading={loadingMore}
          >
            Load More
          </Button>
        </div>
      )}
    </>
  );
}
