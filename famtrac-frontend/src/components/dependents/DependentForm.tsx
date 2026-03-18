import { useState } from 'react';
import { Form } from 'react-bootstrap';
import { Input } from '../common/Input';
import { Button } from '../common/Button';
import { useValidation } from '../../hooks/useValidation';
import { required, minLength, pastDate } from '../../utils/validation';
import type { Dependent } from '../../types/domain';

export interface DependentFormProps {
  dependent?: Dependent;
  familyId: string;
  onSubmit: (data: { name: string; date_of_birth: string; family_id: string }) => Promise<void>;
  onCancel: () => void;
  loading?: boolean;
}

/**
 * DependentForm component for creating and editing dependents
 * - Validates name with minimum 1 character (Requirements 6.2, 8.2)
 * - Validates date of birth is in the past (Requirements 6.3, 8.3)
 * - Requires family_id (Requirement 6.4)
 * - Displays validation errors (Requirement 18.2)
 * - Disables submit button during submission (Requirement 19.2)
 */
export function DependentForm({
  dependent,
  familyId,
  onSubmit,
  onCancel,
  loading = false,
}: DependentFormProps) {
  const [name, setName] = useState(dependent?.name || '');
  const [dateOfBirth, setDateOfBirth] = useState(
    dependent?.date_of_birth ? dependent.date_of_birth.split('T')[0] : ''
  );

  const { validate, validateAll, errors, clearError } = useValidation({
    name: [required('Name'), minLength(1)],
    date_of_birth: [required('Date of birth'), pastDate('Date of birth')],
  });

  const handleSubmit = async (e: React.SubmitEvent) => {
    e.preventDefault();

    const values = {
      name,
      date_of_birth: dateOfBirth,
    };

    const validation = validateAll(values);
    if (!validation.isValid) {
      return;
    }

    await onSubmit({
      name,
      date_of_birth: dateOfBirth,
      family_id: familyId,
    });
  };

  const handleNameBlur = () => {
    validate('name', name);
  };

  const handleDateOfBirthBlur = () => {
    validate('date_of_birth', dateOfBirth);
  };

  const handleNameChange = (value: string) => {
    setName(value);
    if (errors.name) {
      clearError('name');
    }
  };

  const handleDateOfBirthChange = (value: string) => {
    setDateOfBirth(value);
    if (errors.date_of_birth) {
      clearError('date_of_birth');
    }
  };

  const hasErrors = Object.keys(errors).length > 0;
  const isFormValid = name.trim() !== '' && dateOfBirth !== '' && !hasErrors;

  return (
    <Form onSubmit={handleSubmit}>
      <Input
        label="Name"
        value={name}
        onChange={handleNameChange}
        onBlur={handleNameBlur}
        error={errors.name}
        required
        placeholder="Enter dependent's name"
        disabled={loading}
      />

      <Input
        label="Date of Birth"
        type="date"
        value={dateOfBirth}
        onChange={handleDateOfBirthChange}
        onBlur={handleDateOfBirthBlur}
        error={errors.date_of_birth}
        required
        disabled={loading}
      />

      <div className="d-flex gap-2">
        <Button
          type="submit"
          variant="primary"
          disabled={!isFormValid || loading}
          loading={loading}
        >
          {dependent ? 'Update Dependent' : 'Create Dependent'}
        </Button>
        <Button type="button" variant="secondary" onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
      </div>
    </Form>
  );
}
