import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Input } from './Input';

describe('Input', () => {
  it('renders input with label', () => {
    render(<Input label="Name" value="" onChange={() => {}} />);
    expect(screen.getByLabelText(/name/i)).toBeInTheDocument();
  });

  it('shows required indicator when required', () => {
    render(<Input label="Name" value="" onChange={() => {}} required />);
    expect(screen.getByLabelText(/required/i)).toBeInTheDocument();
  });

  it('calls onChange when value changes', async () => {
    const handleChange = vi.fn();
    const user = userEvent.setup();

    render(<Input label="Name" value="" onChange={handleChange} />);

    await user.type(screen.getByLabelText(/name/i), 'John');
    expect(handleChange).toHaveBeenCalled();
  });

  it('calls onBlur when input loses focus', async () => {
    const handleBlur = vi.fn();
    const user = userEvent.setup();

    render(<Input label="Name" value="" onChange={() => {}} onBlur={handleBlur} />);

    const input = screen.getByLabelText(/name/i);
    await user.click(input);
    await user.tab();

    expect(handleBlur).toHaveBeenCalledTimes(1);
  });

  it('displays error message when error prop is provided', () => {
    render(<Input label="Name" value="" onChange={() => {}} error="Name is required" />);

    expect(screen.getByText(/name is required/i)).toBeInTheDocument();
  });

  it('marks input as invalid when error is present', () => {
    render(<Input label="Name" value="" onChange={() => {}} error="Name is required" />);

    const input = screen.getByLabelText(/name/i);
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveClass('border-red-300');
  });

  it('has minimum touch target height', () => {
    render(<Input label="Name" value="" onChange={() => {}} />);
    const input = screen.getByLabelText(/name/i);

    // Check that min-h-[44px] class is present
    expect(input.classList.contains('min-h-[44px]')).toBe(true);
  });

  it('disables input when disabled prop is true', () => {
    render(<Input label="Name" value="" onChange={() => {}} disabled />);
    expect(screen.getByLabelText(/name/i)).toBeDisabled();
  });
});
