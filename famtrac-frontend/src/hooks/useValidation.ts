import { useState, useCallback } from 'react';
import { type ValidationResult } from '../utils/validation';

export type ValidationRule = (value: unknown) => ValidationResult;

export interface FieldValidation {
  [fieldName: string]: ValidationRule[];
}

export interface ValidationErrors {
  [fieldName: string]: string;
}

export interface ValidationResultAll {
  isValid: boolean;
  errors: ValidationErrors;
}

/**
 * Hook for managing form validation with field-level and form-level validation.
 * Tracks validation errors and provides functions to validate and clear errors.
 *
 * @param rules - Object mapping field names to arrays of validation rules
 * @returns Object with validate, validateAll, errors, and clearError functions
 */
export function useValidation(rules: FieldValidation) {
  const [errors, setErrors] = useState<ValidationErrors>({});

  /**
   * Validate a single field and update errors state.
   * Returns the error message if validation fails, null otherwise.
   */
  const validate = useCallback(
    (fieldName: string, value: unknown): string | null => {
      const fieldRules = rules[fieldName];

      if (!fieldRules) {
        return null;
      }

      // Run all validation rules for this field
      for (const rule of fieldRules) {
        const result = rule(value);
        if (!result.isValid) {
          // Update errors state with the first error found
          setErrors((prev) => ({
            ...prev,
            [fieldName]: result.error || 'Validation failed',
          }));
          return result.error || 'Validation failed';
        }
      }

      // All validations passed - clear any existing error
      setErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[fieldName];
        return newErrors;
      });

      return null;
    },
    [rules]
  );

  /**
   * Validate all fields in the form.
   * Returns an object with isValid flag and all errors.
   */
  const validateAll = useCallback(
    (values: Record<string, unknown>): ValidationResultAll => {
      const newErrors: ValidationErrors = {};
      let isValid = true;

      // Validate each field that has rules
      for (const fieldName in rules) {
        const fieldRules = rules[fieldName];
        const value = values[fieldName];

        // Run all validation rules for this field
        for (const rule of fieldRules) {
          const result = rule(value);
          if (!result.isValid) {
            newErrors[fieldName] = result.error || 'Validation failed';
            isValid = false;
            break; // Stop at first error for this field
          }
        }
      }

      setErrors(newErrors);

      return {
        isValid,
        errors: newErrors,
      };
    },
    [rules]
  );

  /**
   * Clear the error for a specific field.
   */
  const clearError = useCallback((fieldName: string) => {
    setErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors[fieldName];
      return newErrors;
    });
  }, []);

  /**
   * Clear all validation errors.
   */
  const clearAllErrors = useCallback(() => {
    setErrors({});
  }, []);

  return {
    validate,
    validateAll,
    errors,
    clearError,
    clearAllErrors,
  };
}
