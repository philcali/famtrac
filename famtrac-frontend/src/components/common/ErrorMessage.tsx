export interface ErrorMessageProps {
  message: string;
  onClose?: () => void;
  dismissible?: boolean;
  className?: string;
}

/**
 * ErrorMessage component
 * - Displays error messages to the user (Requirements 2.5, 4.5, 6.7, etc.)
 */
export function ErrorMessage({
  message,
  onClose,
  dismissible = false,
  className,
}: ErrorMessageProps) {
  return (
    <div
      className={`mb-4 p-4 bg-red-50 border border-red-100 rounded-xl text-red-700 text-sm relative ${className ?? ''}`}
      role="alert"
    >
      {dismissible && onClose && (
        <button onClick={onClose} className="absolute top-2 right-2 text-red-400 hover:text-red-600" aria-label="Dismiss">
          ×
        </button>
      )}
      {message}
    </div>
  );
}
