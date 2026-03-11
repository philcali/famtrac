import { Button as BootstrapButton, Spinner } from 'react-bootstrap';
import type { ReactNode } from 'react';

export interface ButtonProps {
  children: ReactNode;
  onClick?: () => void;
  type?: 'button' | 'submit' | 'reset';
  variant?:
    | 'primary'
    | 'secondary'
    | 'success'
    | 'danger'
    | 'warning'
    | 'info'
    | 'light'
    | 'dark'
    | 'link';
  disabled?: boolean;
  loading?: boolean;
  size?: 'sm' | 'lg';
  className?: string;
}

/**
 * Button component with loading states
 * - Disables button during requests (Requirement 19.2)
 * - Displays loading indicator during requests (Requirement 19.1)
 * - Touch targets at least 44x44 pixels on mobile (Requirement 17.4)
 */
export function Button({
  children,
  onClick,
  type = 'button',
  variant = 'primary',
  disabled = false,
  loading = false,
  size,
  className,
}: ButtonProps) {
  return (
    <BootstrapButton
      type={type}
      variant={variant}
      onClick={onClick}
      disabled={disabled || loading}
      size={size}
      className={className}
      style={{ minHeight: '44px', minWidth: '44px' }} // Ensure 44x44px minimum for touch targets
      aria-busy={loading}
    >
      {loading && (
        <>
          <Spinner
            as="span"
            animation="border"
            size="sm"
            role="status"
            aria-hidden="true"
            className="me-2"
          />
          <span className="visually-hidden">Loading...</span>
        </>
      )}
      {children}
    </BootstrapButton>
  );
}
