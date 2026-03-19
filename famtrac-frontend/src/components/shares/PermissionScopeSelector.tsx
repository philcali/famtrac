import { Form } from 'react-bootstrap';
import type { PermissionAction } from '../../types/domain';
import {
  ALWAYS_REQUIRED,
  PERMISSION_LABELS,
  getLockedActions,
  addActionWithDependencies,
  removeAction,
} from '../../utils/permissions';

export interface PermissionScopeSelectorProps {
  value: PermissionAction[];
  onChange: (actions: PermissionAction[]) => void;
  disabled?: boolean;
}

const ALL_ACTIONS: PermissionAction[] = [
  'family_read',
  'dependent_read',
  'dependent_write',
  'activity_read',
  'activity_write',
];

/**
 * PermissionScopeSelector renders checkboxes for each permission action
 * with dependency enforcement.
 * - family_read always checked and disabled (Requirement 3.1, 3.2)
 * - Auto-selects dependencies when checking an action (Requirement 3.3, 3.4)
 * - Locks dependent actions that are required by selected actions (Requirement 3.5)
 * - Emits selected actions via onChange callback (Requirement 3.6)
 * - Accepts initial value for pre-populating (Requirement 3.7)
 */
export function PermissionScopeSelector({
  value,
  onChange,
  disabled = false,
}: PermissionScopeSelectorProps) {
  const locked = getLockedActions(value);

  const handleToggle = (action: PermissionAction) => {
    if (value.includes(action)) {
      onChange(removeAction(value, action));
    } else {
      onChange(addActionWithDependencies(value, action));
    }
  };

  return (
    <div>
      {ALL_ACTIONS.map((action) => {
        const isAlwaysRequired = ALWAYS_REQUIRED.includes(action);
        const isLocked = locked.has(action);
        const isChecked = value.includes(action) || isAlwaysRequired;
        const isDisabled = disabled || isAlwaysRequired || isLocked;

        return (
          <Form.Check
            key={action}
            type="checkbox"
            id={`permission-${action}`}
            label={PERMISSION_LABELS[action]}
            checked={isChecked}
            disabled={isDisabled}
            onChange={() => handleToggle(action)}
          />
        );
      })}
    </div>
  );
}
