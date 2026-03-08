export interface ValidationResult {
  isValid: boolean;
  error: string | null;
}

export function required(value: unknown): ValidationResult {
  if (value === null || value === undefined || value === '') {
    return { isValid: false, error: 'This field is required' };
  }
  return { isValid: true, error: null };
}

export function minLength(min: number) {
  return (value: string): ValidationResult => {
    if (value.length < min) {
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

export function pastDate(value: string): ValidationResult {
  const date = new Date(value);
  const now = new Date();

  if (isNaN(date.getTime())) {
    return { isValid: false, error: 'Invalid date' };
  }

  if (date >= now) {
    return { isValid: false, error: 'Date must be in the past' };
  }

  return { isValid: true, error: null };
}

export function notFutureDate(value: string): ValidationResult {
  const date = new Date(value);
  const now = new Date();

  if (isNaN(date.getTime())) {
    return { isValid: false, error: 'Invalid date' };
  }

  if (date > now) {
    return { isValid: false, error: 'Date cannot be in the future' };
  }

  return { isValid: true, error: null };
}

export function positiveInteger(value: string | number): ValidationResult {
  const num = typeof value === 'string' ? parseInt(value, 10) : value;

  if (isNaN(num)) {
    return { isValid: false, error: 'Must be a number' };
  }

  if (!Number.isInteger(num)) {
    return { isValid: false, error: 'Must be an integer' };
  }

  if (num <= 0) {
    return { isValid: false, error: 'Must be a positive number' };
  }

  return { isValid: true, error: null };
}

export function dateRange(startDate: string, endDate: string): ValidationResult {
  const start = new Date(startDate);
  const end = new Date(endDate);

  if (isNaN(start.getTime()) || isNaN(end.getTime())) {
    return { isValid: false, error: 'Invalid date range' };
  }

  if (end <= start) {
    return { isValid: false, error: 'End date must be after start date' };
  }

  return { isValid: true, error: null };
}
