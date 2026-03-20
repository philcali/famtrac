import { useState } from 'react';
import { Form, Alert } from 'react-bootstrap';
import { Input } from '../common/Input';
import { Button } from '../common/Button';
import { PermissionScopeSelector } from './PermissionScopeSelector';
import { useValidation } from '../../hooks/useValidation';
import { required } from '../../utils/validation';
import { validatePermissionScope } from '../../utils/permissions';
import type { PermissionAction } from '../../types/domain';
import type { CreateShareRequest } from '../../api/types';

export interface ShareFormProps {
  familyId: string;
  onSubmit: (data: CreateShareRequest) => Promise<void>;
  onCancel: () => void;
  loading?: boolean;
}

/**
 * ShareForm collects accepter email and permission scope to create a share.
 * Follows the DependentForm pattern.
 * - Email input with required and email validators (Requirement 4.1, 4.2)
 * - Embeds PermissionScopeSelector (Requirement 4.3)
 * - Validates permission scope before submission (Requirement 11.1, 11.2, 11.3, 11.4)
 * - Calls onSubmit with CreateShareRequest data (Requirement 4.4)
 */
export function ShareForm({ onSubmit, onCancel, loading = false }: ShareFormProps) {
  const [accepterUsername, setAccepterUsername] = useState('');
  const [selectedActions, setSelectedActions] = useState<PermissionAction[]>(['family_read']);
  const [scopeError, setScopeError] = useState<string | null>(null);

  const { validate, validateAll, errors, clearError } = useValidation({
    username: [required('Username')],
  });

  const handleSubmit = async (e: React.SubmitEvent) => {
    e.preventDefault();

    const validation = validateAll({ email: accepterUsername });
    if (!validation.isValid) {
      return;
    }

    const permError = validatePermissionScope(selectedActions);
    if (permError) {
      setScopeError(permError);
      return;
    }

    setScopeError(null);
    await onSubmit({
      accepter_username: accepterUsername,
      permission_scope: { actions: selectedActions },
    });
  };

  const handleUsernameChange = (value: string) => {
    setAccepterUsername(value);
    if (errors.username) {
      clearError('username');
    }
  };

  const handleUsernameBlur = () => {
    validate('username', accepterUsername);
  };

  const handleActionsChange = (actions: PermissionAction[]) => {
    setSelectedActions(actions);
    if (scopeError) {
      setScopeError(null);
    }
  };

  const hasErrors = Object.keys(errors).length > 0 || scopeError !== null;
  const isFormValid = accepterUsername.trim() !== '' && !hasErrors;

  return (
    <Form onSubmit={handleSubmit}>
      <Input
        label="Username"
        value={accepterUsername}
        onChange={handleUsernameChange}
        error={errors.email}
        onBlur={handleUsernameBlur}
        required
        placeholder="Enter accepter's username"
        disabled={loading}
      />

      <Form.Group className="mb-3">
        <Form.Label>Permissions</Form.Label>
        <PermissionScopeSelector
          value={selectedActions}
          onChange={handleActionsChange}
          disabled={loading}
        />
        {scopeError && (
          <Alert variant="danger" className="mt-2">
            {scopeError}
          </Alert>
        )}
      </Form.Group>

      <div className="d-flex gap-2">
        <Button
          type="submit"
          variant="primary"
          disabled={!isFormValid || loading}
          loading={loading}
        >
          Create Share
        </Button>
        <Button type="button" variant="secondary" onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
      </div>
    </Form>
  );
}
