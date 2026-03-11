import { Alert } from 'react-bootstrap';

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
    <Alert variant="danger" onClose={onClose} dismissible={dismissible} className={className}>
      {message}
    </Alert>
  );
}
