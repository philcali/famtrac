import { Badge } from '../common/Badge';
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
          variant={permissionLabel.match(/Edit/) ? 'warning' : 'primary'}
          key={permissionLabel.replace(' ', '-')}
        >
          {permissionLabel}
        </Badge>
      ))}
    </>
  );
}
