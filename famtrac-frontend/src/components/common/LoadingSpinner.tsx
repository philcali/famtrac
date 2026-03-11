import { Spinner } from 'react-bootstrap';

export interface LoadingSpinnerProps {
  size?: 'sm';
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark';
  className?: string;
  text?: string;
}

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
  return (
    <div className={`d-flex align-items-center ${className || ''}`}>
      <Spinner animation="border" variant={variant} size={size} role="status" aria-label={text} />
      {text && <span className="ms-2">{text}</span>}
    </div>
  );
}
