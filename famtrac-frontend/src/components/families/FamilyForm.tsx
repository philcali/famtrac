import { useState } from 'react';
import { Input } from '../common/Input';
import { Button } from '../common/Button';
import { useValidation } from '../../hooks/useValidation';
import type { Family } from '../../types/domain';
import { minLength, required } from '../../utils/validation';

export interface FamilyFormProps {
  family?: Family;
  onSubmit: (name: string) => Promise<void>;
  onCancel: () => void;
  loading?: boolean;
}

/**
 * FamilyForm component for creating and editing families
 * - Validates family name (minimum 1 character) (Requirements 2.2, 4.2)
 * - Displays validation errors (Requirement 18.2)
 * - Disables submit button with validation errors (Requirement 18.4)
 * - Triggers validation on blur (Requirement 18.1)
 */
export function FamilyForm({ family, onSubmit, onCancel, loading = false }: FamilyFormProps) {
  const [name, setName] = useState(family?.name || '');

  const { validate, validateAll, errors, clearError } = useValidation({
    name: [required('Name'), minLength(1)],
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validate all fields
    const validation = validateAll({ name });
    if (!validation.isValid) {
      return;
    }

    await onSubmit(name);
  };

  const handleNameBlur = () => {
    validate('name', name);
  };

  const handleNameChange = (value: string) => {
    setName(value);
    if (errors.name) {
      clearError('name');
    }
  };

  const isFormValid = name.length >= 1 && !errors.name;

  return (
    <form onSubmit={handleSubmit}>
      <Input
        label="Family Name"
        value={name}
        onChange={handleNameChange}
        onBlur={handleNameBlur}
        error={errors.name}
        required
        placeholder="Enter family name"
        disabled={loading}
      />

      <div className="flex gap-2 justify-end mt-3">
        <Button variant="secondary" onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
        <Button
          type="submit"
          variant="primary"
          disabled={!isFormValid || loading}
          loading={loading}
        >
          {family ? 'Update' : 'Create'} Family
        </Button>
      </div>
    </form>
  );
}
