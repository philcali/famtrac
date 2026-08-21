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
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <SkeletonCard count={3} />
      </div>
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
      <div className="overflow-x-auto">
        <table className="w-full text-sm text-left">
          <thead>
            <tr className="border-b border-gray-200">
              <th className="py-2 pr-4 font-medium text-gray-500">Username</th>
              <th className="py-2 pr-4 font-medium text-gray-500">Status</th>
              <th className="py-2 pr-4 font-medium text-gray-500">Permissions</th>
              <th className="py-2 font-medium text-gray-500">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {shares.map((share) => (
              <tr key={share.id} className="align-top">
                <td className="py-3 pr-4 align-top text-sm font-medium">{share.accepter_username}</td>
                <td className="py-3 pr-4 align-top">
                  <ShareStatusBadge status={share.status} />
                </td>
                <td className="py-3 pr-4 align-top text-sm">
                  <SharePermissionBadges share={share} />
                </td>
                <td className="py-3 align-top">
                  <div className="flex gap-1">
                    <Button
                      variant="secondary"
                      size="sm"
                      icon="pencil"
                      onClick={() => onEdit(share)}
                      disabled={share.status === 'expired'}
                    ></Button>
                    <Button
                      icon="trash"
                      variant="danger"
                      size="sm"
                      onClick={() => onRevoke(share)}
                    ></Button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
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
