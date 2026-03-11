import type { ValidationRule } from '../hooks/useValidation';

export interface ValidationResult {
  isValid: boolean;
  error: string | null;
}

export function required(field: string): ValidationRule {
  return (value: unknown): ValidationResult => {
    if (value === null || value === undefined || value === '') {
      return { isValid: false, error: `${field} is required` };
    }
    return { isValid: true, error: null };
  };
}

export function minLength(min: number): ValidationRule {
  return (value: unknown): ValidationResult => {
    if (typeof value === 'string' && (value as string).length < min) {
      return { isValid: false, error: `Must be at least ${min} character${min !== 1 ? 's' : ''}` };
    }
    return { isValid: true, error: null };
  };
}

export function maxLength(max: number) {
  return (value: string): ValidationResult => {
    if (value.length > max) {
      return { isValid: false, error: `Must be at most ${max} character${max !== 1 ? 's' : ''}` };
    }
    return { isValid: true, error: null };
  };
}

export function pattern(regex: RegExp, message: string) {
  return (value: string): ValidationResult => {
    if (!regex.test(value)) {
      return { isValid: false, error: message };
    }
    return { isValid: true, error: null };
  };
}

export function pastDate(field: string): ValidationRule {
  return (value: unknown) => {
    if (typeof value !== 'string' && typeof value !== 'number') {
      return { isValid: false, error: 'Invalid date' };
    }

    const date = new Date(value);
    const now = new Date();

    if (isNaN(date.getTime())) {
      return { isValid: false, error: 'Invalid date' };
    }

    if (date >= now) {
      return { isValid: false, error: `${field} must be in the past` };
    }

    return { isValid: true, error: null };
  };
}

export function notFutureDate(field: string): ValidationRule {
  return (value: unknown) => {
    if (typeof value !== 'string' && typeof value !== 'number') {
      return { isValid: false, error: 'Invalid date' };
    }
    const date = new Date(value);
    const now = new Date();

    if (isNaN(date.getTime())) {
      return { isValid: false, error: 'Invalid date' };
    }

    if (date > now) {
      return { isValid: false, error: `${field} cannot be in the future` };
    }

    return { isValid: true, error: null };
  };
}

export function positiveInteger(field: string): ValidationRule {
  return (value: unknown) => {
    if (typeof value !== 'string' && typeof value !== 'number') {
      return { isValid: false, error: 'Invalid number' };
    }

    const num = typeof value === 'string' ? parseInt(value, 10) : value;

    if (isNaN(num)) {
      return { isValid: false, error: `${field} must be a number` };
    }

    if (!Number.isInteger(num)) {
      return { isValid: false, error: `${field} must be an integer` };
    }

    if (num <= 0) {
      return { isValid: false, error: `${field} be a positive number` };
    }

    return { isValid: true, error: null };
  };
}

export function dateRange(startDate: string, endDate: string): ValidationRule {
  return (value: unknown) => {
    if (typeof value !== 'string' && typeof value !== 'number') {
      return { isValid: false, error: 'Invalid date' };
    }
    const start = new Date(startDate);
    const end = new Date(endDate);
    const current = new Date(value);

    if (isNaN(start.getTime()) || isNaN(end.getTime())) {
      return { isValid: false, error: 'Invalid date range' };
    }

    if (end <= start) {
      return { isValid: false, error: 'End date must be after start date' };
    }

    if (current < start || current > end) {
      return { isValid: false, error: `Date is not within ${start} and ${end}` };
    }

    return { isValid: true, error: null };
  };
}
