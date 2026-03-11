import { Form } from 'react-bootstrap';

export interface InputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  onBlur?: () => void;
  error?: string;
  required?: boolean;
  type?: string;
  id?: string;
  placeholder?: string;
  disabled?: boolean;
}

/**
 * Input component with validation support
 * - Displays validation errors below the field (Requirement 18.2)
 * - Removes validation errors when field passes (Requirement 18.3)
 * - Marks required fields with visual indicator (Requirement 18.6)
 * - Touch targets at least 44x44 pixels on mobile (Requirement 17.4)
 */
export function Input({
  label,
  value,
  onChange,
  onBlur,
  error,
  required = false,
  type = 'text',
  id,
  placeholder,
  disabled = false,
}: InputProps) {
  const inputId = id || `input-${label.toLowerCase().replace(/\s+/g, '-')}`;

  return (
    <Form.Group className="mb-3">
      <Form.Label htmlFor={inputId}>
        {label}
        {required && (
          <span className="text-danger ms-1" aria-label="required">
            *
          </span>
        )}
      </Form.Label>
      <Form.Control
        id={inputId}
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={onBlur}
        isInvalid={!!error}
        placeholder={placeholder}
        disabled={disabled}
        aria-invalid={!!error}
        aria-describedby={error ? `${inputId}-error` : undefined}
        aria-required={required}
        style={{ minHeight: '44px' }} // Ensure 44px minimum height for touch targets
      />
      {error && (
        <Form.Control.Feedback type="invalid" id={`${inputId}-error`}>
          {error}
        </Form.Control.Feedback>
      )}
    </Form.Group>
  );
}
