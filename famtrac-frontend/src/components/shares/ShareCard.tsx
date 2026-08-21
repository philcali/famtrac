import { Button } from '../common/Button';
import { ShareStatusBadge } from './ShareStatusBadge';
import { PERMISSION_LABELS } from '../../utils/permissions';
import type { Share } from '../../types/domain';
import type { PermissionAction } from '../../types/domain';

export interface ShareCardProps {
  share: Share;
  onEdit?: (share: Share) => void;
  onRevoke?: (share: Share) => void;
  onAccept?: (share: Share) => void;
}

/**
 * ShareCard displays a single share with status, permissions, and action buttons.
 * Follows the DependentCard pattern.
 * - Displays accepter email (Requirement 5.1)
 * - Displays ShareStatusBadge (Requirement 5.2)
 * - Displays permission labels (Requirement 5.3)
 * - Edit button calls onEdit, disabled when expired (Requirement 5.4, 5.6)
 * - Revoke button calls onRevoke (Requirement 5.5)
 * - Accept button only when onAccept provided (for PendingSharesPage)
 */
export function ShareCard({ share, onEdit, onRevoke, onAccept }: ShareCardProps) {
  const permissionLabels = share.permission_scope.actions
    .map((action) => PERMISSION_LABELS[action as PermissionAction])
    .filter(Boolean);

  return (
    <div className="mb-3 p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
      <div className="flex justify-between items-center mb-2">
        <span className="text-base font-semibold">{share.accepter_username}</span>
        <ShareStatusBadge status={share.status} />
      </div>
      <div className="text-sm mb-2">
        <strong>Permissions:</strong> {permissionLabels.join(', ')}
      </div>
      <div className="flex gap-2">
        {onEdit && (
          <Button
            variant="secondary"
            size="sm"
            icon="pencil"
            onClick={() => onEdit(share)}
            disabled={share.status === 'expired'}
          >
            Edit
          </Button>
        )}
        {onRevoke && (
          <Button variant="danger" size="sm" icon="trash" onClick={() => onRevoke(share)}>
            Revoke
          </Button>
        )}
        {onAccept && (
          <Button variant="success" size="sm" icon="plus" onClick={() => onAccept(share)}>
            Accept
          </Button>
        )}
      </div>
    </div>
  );
}
