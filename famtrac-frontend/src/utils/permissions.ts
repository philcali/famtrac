import type { PermissionAction } from '../types/domain';

/** Actions that are always required and cannot be unchecked */
export const ALWAYS_REQUIRED: PermissionAction[] = ['family_read'];

/** Map of actions to their required dependencies */
export const PERMISSION_DEPENDENCIES: Record<PermissionAction, PermissionAction[]> = {
  family_read: [],
  dependent_read: [],
  dependent_write: ['dependent_read'],
  activity_read: [],
  activity_write: ['activity_read', 'dependent_read'],
  recipe_read: [],
  recipe_write: ['recipe_read'],
  meal_slot_read: [],
  meal_slot_write: ['meal_slot_read'],
  feeding_log_read: [],
  feeding_log_write: ['feeding_log_read'],
};

/** Human-readable labels for each permission action */
export const PERMISSION_LABELS: Record<PermissionAction, string> = {
  family_read: 'View Family',
  dependent_read: 'View Dependents',
  dependent_write: 'Edit Dependents',
  activity_read: 'View Activities',
  activity_write: 'Edit Activities',
  recipe_read: 'View Recipes',
  recipe_write: 'Edit Recipes',
  meal_slot_read: 'View Meal Plan',
  meal_slot_write: 'Edit Meal Plan',
  feeding_log_read: 'View Feeding Logs',
  feeding_log_write: 'Edit Feeding Logs',
};

/**
 * Given the currently selected actions, compute which actions
 * are forced on (required by a dependency of another selected action)
 * and cannot be unchecked.
 */
export function getLockedActions(selected: PermissionAction[]): Set<PermissionAction> {
  const locked = new Set<PermissionAction>(ALWAYS_REQUIRED);
  for (const action of selected) {
    for (const dep of PERMISSION_DEPENDENCIES[action]) {
      locked.add(dep);
    }
  }
  return locked;
}

/**
 * When an action is toggled on, return the new set of selected actions
 * including any dependencies that must be auto-selected.
 */
export function addActionWithDependencies(
  current: PermissionAction[],
  action: PermissionAction
): PermissionAction[] {
  const result = new Set<PermissionAction>(current);
  result.add(action);
  for (const dep of PERMISSION_DEPENDENCIES[action]) {
    result.add(dep);
  }
  return Array.from(result);
}

/**
 * When an action is toggled off, return the new set of selected actions.
 * Also removes actions that depended on the removed action, unless
 * another selected action also requires them.
 */
export function removeAction(
  current: PermissionAction[],
  action: PermissionAction
): PermissionAction[] {
  // Never allow removing always-required actions
  if (ALWAYS_REQUIRED.includes(action)) {
    return [...current];
  }

  let result = current.filter((a) => a !== action);

  // Cascade: remove any action whose dependency on the removed action
  // is no longer satisfied. Repeat until stable since removals can cascade.
  let changed = true;
  while (changed) {
    changed = false;
    for (const remaining of [...result]) {
      if (ALWAYS_REQUIRED.includes(remaining)) continue;
      const deps = PERMISSION_DEPENDENCIES[remaining];
      const hasUnmetDep = deps.some((dep) => !result.includes(dep));
      if (hasUnmetDep) {
        result = result.filter((a) => a !== remaining);
        changed = true;
        break;
      }
    }
  }

  return result;
}

/**
 * Validate a permission scope. Returns null if valid, or an error message string.
 */
export function validatePermissionScope(actions: PermissionAction[]): string | null {
  if (!actions.includes('family_read')) {
    return 'family_read is required';
  }
  if (actions.includes('dependent_write') && !actions.includes('dependent_read')) {
    return 'dependent_write requires dependent_read';
  }
  if (actions.includes('activity_write')) {
    if (!actions.includes('activity_read')) {
      return 'activity_write requires activity_read';
    }
    if (!actions.includes('dependent_read')) {
      return 'activity_write requires dependent_read';
    }
  }
  if (actions.includes('recipe_write') && !actions.includes('recipe_read')) {
    return 'recipe_write requires recipe_read';
  }
  if (actions.includes('meal_slot_write') && !actions.includes('meal_slot_read')) {
    return 'meal_slot_write requires meal_slot_read';
  }
  if (actions.includes('feeding_log_write') && !actions.includes('feeding_log_read')) {
    return 'feeding_log_write requires feeding_log_read';
  }
  return null;
}
