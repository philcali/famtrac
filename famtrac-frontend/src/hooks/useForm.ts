import { useState, useCallback, type ChangeEvent } from 'react';
import { useValidation, type FieldValidation, type ValidationErrors } from './useValidation';

export interface UseFormOptions<T> {
  initialValues: T;
  validationRules?: FieldValidation;
  onSubmit: (values: T) => void | Promise<void>;
}

export interface UseFormReturn<T> {
  values: T;
  errors: ValidationErrors;
  isSubmitting: boolean;
  handleChange: (
    e: ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>
  ) => void;
  handleBlur: (e: ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => void;
  handleSubmit: (e: React.FormEvent) => Promise<void>;
  setFieldValue: (fieldName: keyof T, value: unknown) => void;
  resetForm: () => void;
  isValid: boolean;
}

/**
 * Hook for managing form state with integrated validation.
 * Handles form values, validation, submission, and provides helper functions.
 *
 * @param options - Configuration object with initialValues, validationRules, and onSubmit
 * @returns Object with form state and handler functions
 */
export function useForm<T extends Record<string, unknown>>(
  options: UseFormOptions<T>
): UseFormReturn<T> {
  const { initialValues, validationRules = {}, onSubmit } = options;

  const [values, setValues] = useState<T>(initialValues);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const { validate, validateAll, errors, clearAllErrors } = useValidation(validationRules);

  /**
   * Handle input change events.
   * Updates the form values state.
   */
  const handleChange = useCallback(
    (e: ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
      const { name, value, type } = e.target;

      // Handle checkbox inputs
      if (type === 'checkbox') {
        const checked = (e.target as HTMLInputElement).checked;
        setValues((prev) => ({
          ...prev,
          [name]: checked,
        }));
      } else {
        setValues((prev) => ({
          ...prev,
          [name]: value,
        }));
      }
    },
    []
  );

  /**
   * Handle input blur events.
   * Triggers validation for the field that lost focus.
   */
  const handleBlur = useCallback(
    (e: ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
      const { name, value } = e.target;
      validate(name, value);
    },
    [validate]
  );

  /**
   * Handle form submission.
   * Validates all fields and calls onSubmit if valid.
   */
  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();

      // Validate all fields
      const validationResult = validateAll(values);

      if (!validationResult.isValid) {
        // Don't submit if validation fails
        return;
      }

      // Submit the form
      setIsSubmitting(true);
      try {
        await onSubmit(values);
      } finally {
        setIsSubmitting(false);
      }
    },
    [values, validateAll, onSubmit]
  );

  /**
   * Set a specific field value programmatically.
   */
  const setFieldValue = useCallback((fieldName: keyof T, value: unknown) => {
    setValues((prev) => ({
      ...prev,
      [fieldName]: value,
    }));
  }, []);

  /**
   * Reset form to initial values and clear all errors.
   */
  const resetForm = useCallback(() => {
    setValues(initialValues);
    clearAllErrors();
    setIsSubmitting(false);
  }, [initialValues, clearAllErrors]);

  /**
   * Check if form is valid (no errors).
   */
  const isValid = Object.keys(errors).length === 0;

  return {
    values,
    errors,
    isSubmitting,
    handleChange,
    handleBlur,
    handleSubmit,
    setFieldValue,
    resetForm,
    isValid,
  };
}
