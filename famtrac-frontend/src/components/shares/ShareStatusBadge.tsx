import { Badge } from 'react-bootstrap';
import type { ShareStatus } from '../../types/domain';

export interface ShareStatusBadgeProps {
  status: ShareStatus;
}

const STATUS_VARIANT: Record<ShareStatus, string> = {
  pending: 'warning',
  active: 'success',
  expired: 'secondary',
};

/**
 * ShareStatusBadge renders a colored badge for share status.
 * - pending → warning (yellow), text "Pending" (Requirement 10.1)
 * - active → success (green), text "Active" (Requirement 10.2)
 * - expired → secondary (gray), text "Expired" (Requirement 10.3)
 */
export function ShareStatusBadge({ status }: ShareStatusBadgeProps) {
  const variant = STATUS_VARIANT[status];
  const label = status.charAt(0).toUpperCase() + status.slice(1);

  return <Badge bg={variant}>{label}</Badge>;
}
