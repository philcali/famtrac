import type { ReactNode } from 'react';

const VARIANT_CLASSES: Record<string, string> = {
  primary: 'bg-blue-100 text-blue-700',
  secondary: 'bg-gray-100 text-gray-700',
  success: 'bg-green-100 text-green-700',
  danger: 'bg-red-100 text-red-700',
  warning: 'bg-yellow-100 text-yellow-700',
  info: 'bg-sky-100 text-sky-700',
  light: 'bg-gray-100 text-gray-700',
  dark: 'bg-gray-800 text-white',
};

export interface BadgeProps {
  variant?: string;
  children: ReactNode;
  className?: string;
}

export function Badge({ variant = 'secondary', children, className = '' }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${VARIANT_CLASSES[variant] || VARIANT_CLASSES.secondary} ${className}`}
    >
      {children}
    </span>
  );
}
