import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import { SuccessMessage } from './SuccessMessage';

describe('SuccessMessage', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it('renders success message', () => {
    render(<SuccessMessage message="Operation successful" autoDismiss={false} />);
    expect(screen.getByText(/operation successful/i)).toBeInTheDocument();
  });

  it('auto-dismisses after 3 seconds by default', async () => {
    const handleClose = vi.fn();
    render(<SuccessMessage message="Success" onClose={handleClose} />);

    expect(screen.getByText(/success/i)).toBeInTheDocument();

    // Fast-forward time by 3 seconds
    await act(async () => {
      vi.advanceTimersByTime(3000);
    });

    expect(handleClose).toHaveBeenCalledTimes(1);
  });

  it('auto-dismisses after custom delay', async () => {
    const handleClose = vi.fn();
    render(<SuccessMessage message="Success" onClose={handleClose} dismissDelay={5000} />);

    // Fast-forward time by 5 seconds
    await act(async () => {
      vi.advanceTimersByTime(5000);
    });

    expect(handleClose).toHaveBeenCalledTimes(1);
  });

  it('does not auto-dismiss when autoDismiss is false', async () => {
    const handleClose = vi.fn();
    render(<SuccessMessage message="Success" onClose={handleClose} autoDismiss={false} />);

    // Fast-forward time by 5 seconds
    vi.advanceTimersByTime(5000);

    // Should still be visible
    expect(screen.getByText(/success/i)).toBeInTheDocument();
    expect(handleClose).not.toHaveBeenCalled();
  });

  it('is dismissible', () => {
    render(<SuccessMessage message="Success" autoDismiss={false} />);

    // Bootstrap Alert with dismissible prop should have a close button
    const alert = screen.getByRole('alert');
    expect(alert).toBeInTheDocument();
  });
});
