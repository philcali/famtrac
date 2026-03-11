import { Modal } from 'react-bootstrap';
import { Button } from './Button';

export interface ConfirmDialogProps {
  show: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  confirmVariant?:
    | 'primary'
    | 'secondary'
    | 'success'
    | 'danger'
    | 'warning'
    | 'info'
    | 'light'
    | 'dark';
  onConfirm: () => void;
  onCancel: () => void;
  loading?: boolean;
}

/**
 * ConfirmDialog component
 * - Displays confirmation dialogs for deletions (Requirements 2.6, 5.2, 9.2, 13.2)
 */
export function ConfirmDialog({
  show,
  title,
  message,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  confirmVariant = 'danger',
  onConfirm,
  onCancel,
  loading = false,
}: ConfirmDialogProps) {
  return (
    <Modal show={show} onHide={onCancel} centered>
      <Modal.Header closeButton>
        <Modal.Title>{title}</Modal.Title>
      </Modal.Header>
      <Modal.Body>{message}</Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" onClick={onCancel} disabled={loading}>
          {cancelText}
        </Button>
        <Button variant={confirmVariant} onClick={onConfirm} loading={loading}>
          {confirmText}
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
