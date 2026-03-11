import { Alert } from 'react-bootstrap';
import { useEffect, useState } from 'react';

export interface SuccessMessageProps {
  message: string;
  onClose?: () => void;
  autoDismiss?: boolean;
  dismissDelay?: number;
  className?: string;
}

/**
 * SuccessMessage component
 * - Displays success messages for operations (Requirement 19.4)
 * - Auto-dismisses success messages after 3 seconds (Requirement 19.5)
 */
export function SuccessMessage({
  message,
  onClose,
  autoDismiss = true,
  dismissDelay = 3000,
  className,
}: SuccessMessageProps) {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    if (autoDismiss && visible) {
      const timer = setTimeout(() => {
        setVisible(false);
        if (onClose) {
          onClose();
        }
      }, dismissDelay);

      return () => clearTimeout(timer);
    }
  }, [autoDismiss, dismissDelay, onClose, visible]);

  if (!visible) {
    return null;
  }

  return (
    <Alert
      variant="success"
      onClose={() => {
        setVisible(false);
        if (onClose) {
          onClose();
        }
      }}
      dismissible
      className={className}
    >
      {message}
    </Alert>
  );
}
