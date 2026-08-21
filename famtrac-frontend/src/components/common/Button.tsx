import type { ReactNode } from 'react';
import { Icon } from './Icon';

export interface ButtonProps {
  children?: ReactNode;
  onClick?: () => void;
  type?: 'button' | 'submit' | 'reset';
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark' | 'link' | 'outline-primary' | 'outline-secondary' | 'outline-success' | 'outline-danger' | 'outline-warning' | 'outline-info';
  disabled?: boolean;
  loading?: boolean;
  size?: 'sm' | 'lg';
  className?: string;
  icon?: string;
}

const VARIANT_CLASSES: Record<string, string> = {
  primary: 'bg-blue-600 text-white hover:bg-blue-700 active:bg-blue-800',
  secondary: 'bg-gray-100 text-gray-700 hover:bg-gray-200 active:bg-gray-300 border border-gray-200',
  success: 'bg-green-600 text-white hover:bg-green-700 active:bg-green-800',
  danger: 'bg-red-600 text-white hover:bg-red-700 active:bg-red-800',
  warning: 'bg-yellow-500 text-white hover:bg-yellow-600 active:bg-yellow-700',
  info: 'bg-sky-500 text-white hover:bg-sky-600 active:bg-sky-700',
  light: 'bg-gray-50 text-gray-700 hover:bg-gray-100 active:bg-gray-200 border border-gray-200',
  dark: 'bg-gray-800 text-white hover:bg-gray-900 active:bg-gray-950',
  link: 'bg-transparent text-blue-600 hover:underline p-0 min-w-0 min-h-0',
  'outline-primary': 'bg-transparent text-blue-600 border border-blue-600 hover:bg-blue-50 active:bg-blue-100',
  'outline-secondary': 'bg-transparent text-gray-600 border border-gray-600 hover:bg-gray-50 active:bg-gray-100',
  'outline-success': 'bg-transparent text-green-600 border border-green-600 hover:bg-green-50 active:bg-green-100',
  'outline-danger': 'bg-transparent text-red-600 border border-red-600 hover:bg-red-50 active:bg-red-100',
  'outline-warning': 'bg-transparent text-yellow-600 border border-yellow-600 hover:bg-yellow-50 active:bg-yellow-100',
  'outline-info': 'bg-transparent text-sky-600 border border-sky-600 hover:bg-sky-50 active:bg-sky-100',
};

const SIZE_CLASSES: Record<string, string> = {
  sm: 'text-sm px-3 py-1.5',
  lg: 'text-lg px-5 py-3',
};

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
  icon,
}: ButtonProps) {
  const base = 'inline-flex items-center justify-center rounded-xl font-medium transition-colors active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none';
  const variantCls = VARIANT_CLASSES[variant] || VARIANT_CLASSES.primary;
  const sizeCls = size ? (SIZE_CLASSES[size] || '') : 'px-4 py-2.5';
  const touch = variant !== 'link' ? 'min-h-[44px] min-w-[44px]' : '';

  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled || loading}
      className={`${base} ${variantCls} ${sizeCls} ${touch} ${className ?? ''}`}
      aria-busy={loading}
    >
      {loading && (
        <span className="inline-block w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full animate-spin" role="status" aria-hidden="true">
          <span className="sr-only">Loading...</span>
        </span>
      )}
      {icon && <Icon name={icon} size={14} />}
      {children}
    </button>
  );
}
