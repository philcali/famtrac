import type { JSX } from 'react';
import { getIcon } from '../../utils/iconRegistry';

export interface IconProps {
  name: string;
  size?: number;
  className?: string;
}

export function Icon({ name, size = 16, className }: IconProps): JSX.Element | null {
  const svg = getIcon(name);
  if (!svg) return null;

  return (
    <span
      aria-hidden="true"
      className={className}
      style={{ display: 'inline-flex', width: size, height: size }}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}
