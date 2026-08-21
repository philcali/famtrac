export interface LoadingSpinnerProps {
  size?: 'sm';
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark';
  className?: string;
  text?: string;
}

const VARIANT_COLORS: Record<string, string> = {
  primary: 'border-blue-600',
  secondary: 'border-gray-400',
  success: 'border-green-600',
  danger: 'border-red-600',
  warning: 'border-yellow-500',
  info: 'border-sky-500',
  light: 'border-gray-200',
  dark: 'border-gray-800',
};

/**
 * LoadingSpinner component
 * - Displays loading indicator during requests (Requirement 19.1)
 */
export function LoadingSpinner({
  size,
  variant = 'primary',
  className,
  text = 'Loading...',
}: LoadingSpinnerProps) {
  const color = VARIANT_COLORS[variant] || VARIANT_COLORS.primary;
  const spinnerSize = size === 'sm' ? 'w-4 h-4 border-2' : 'w-8 h-8 border-4';

  return (
    <div className={`flex items-center ${className ?? ''}`}>
      <span
        className={`inline-block ${spinnerSize} border-current border-t-transparent rounded-full animate-spin ${color}`}
        role="status"
        aria-label={text}
      />
      {text && <span className="ml-2">{text}</span>}
    </div>
  );
}
