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
    <div className="mb-3">
      <label htmlFor={inputId} className="block text-sm font-medium text-gray-700 mb-1">
        {label}
        {required && (
          <span className="text-red-500 ml-0.5" aria-label="required">
            *
          </span>
        )}
      </label>
      <input
        id={inputId}
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={onBlur}
        placeholder={placeholder}
        disabled={disabled}
        aria-invalid={!!error}
        aria-describedby={error ? `${inputId}-error` : undefined}
        aria-required={required}
        className={`w-full px-3 py-2.5 rounded-xl border border-gray-200 bg-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/30 focus:border-blue-400 transition-colors min-h-[44px] ${
          error ? 'border-red-300 bg-red-50' : ''
        } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
      />
      {error && (
        <p id={`${inputId}-error`} className="mt-1 text-xs text-red-500" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
