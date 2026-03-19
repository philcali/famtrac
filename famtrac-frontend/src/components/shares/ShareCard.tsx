import { Card } from 'react-bootstrap';
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
    <Card className="mb-3">
      <Card.Body>
        <Card.Title className="d-flex justify-content-between align-items-center">
          <span>{share.accepter_email}</span>
          <ShareStatusBadge status={share.status} />
        </Card.Title>
        <Card.Text>
          <strong>Permissions:</strong> {permissionLabels.join(', ')}
        </Card.Text>
        <div className="d-flex gap-2">
          {onEdit && (
            <Button
              variant="secondary"
              size="sm"
              onClick={() => onEdit(share)}
              disabled={share.status === 'expired'}
            >
              Edit
            </Button>
          )}
          {onRevoke && (
            <Button variant="danger" size="sm" onClick={() => onRevoke(share)}>
              Revoke
            </Button>
          )}
          {onAccept && (
            <Button variant="success" size="sm" onClick={() => onAccept(share)}>
              Accept
            </Button>
          )}
        </div>
      </Card.Body>
    </Card>
  );
}
