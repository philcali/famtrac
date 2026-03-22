import { Badge } from 'react-bootstrap';
import type { PermissionAction, Share } from '../../types/domain';
import { PERMISSION_LABELS } from '../../utils/permissions';

export interface SharePermissionBadgesProps {
  share: Share;
}

export function SharePermissionBadges({ share }: SharePermissionBadgesProps) {
  const permissionLabels = share.permission_scope.actions
    .map((action) => PERMISSION_LABELS[action as PermissionAction])
    .filter(Boolean);
  return (
    <>
      {permissionLabels.map((permissionLabel) => (
        <Badge
          bg={permissionLabel.match(/Edit/) ? 'warning' : 'primary'}
          className="me-1"
          key={permissionLabel.replace(' ', '-')}
          pill
        >
          {permissionLabel}
        </Badge>
      ))}
    </>
  );
}
