import { useState } from 'react';
import { Form, Alert } from 'react-bootstrap';
import { Input } from '../common/Input';
import { Button } from '../common/Button';
import { PermissionScopeSelector } from './PermissionScopeSelector';
import { useValidation } from '../../hooks/useValidation';
import { required, email } from '../../utils/validation';
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
  const [accepterEmail, setAccepterEmail] = useState('');
  const [selectedActions, setSelectedActions] = useState<PermissionAction[]>(['family_read']);
  const [scopeError, setScopeError] = useState<string | null>(null);

  const { validate, validateAll, errors, clearError } = useValidation({
    email: [required('Email'), email('Email')],
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const validation = validateAll({ email: accepterEmail });
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
      accepter_email: accepterEmail,
      permission_scope: { actions: selectedActions },
    });
  };

  const handleEmailBlur = () => {
    validate('email', accepterEmail);
  };

  const handleEmailChange = (value: string) => {
    setAccepterEmail(value);
    if (errors.email) {
      clearError('email');
    }
  };

  const handleActionsChange = (actions: PermissionAction[]) => {
    setSelectedActions(actions);
    if (scopeError) {
      setScopeError(null);
    }
  };

  const hasErrors = Object.keys(errors).length > 0 || scopeError !== null;
  const isFormValid = accepterEmail.trim() !== '' && !hasErrors;

  return (
    <Form onSubmit={handleSubmit}>
      <Input
        label="Email"
        type="email"
        value={accepterEmail}
        onChange={handleEmailChange}
        onBlur={handleEmailBlur}
        error={errors.email}
        required
        placeholder="Enter accepter's email"
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
