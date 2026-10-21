import type { ReactNode, Ref } from 'react';
import type { Rgb } from '../../../lib/cover-palette';

interface EmbedShellProps {
  children: ReactNode;
  ref?: Ref<HTMLDivElement>;
  tint?: Rgb | null;
  mine?: boolean;
  dim?: boolean;
  compact?: boolean;
  label?: string;
}

function background(tint: Rgb | null, mine: boolean): string {
  const base = mine ? 'rgba(8,8,11,0.32)' : 'rgba(255,255,255,0.035)';
  if (!tint) return base;
  const [r, g, b] = tint;
  return `linear-gradient(128deg, rgba(${r},${g},${b},0.22) 0%, rgba(${r},${g},${b},0.07) 46%, transparent 100%), ${base}`;
}

export function EmbedShell({
  children,
  ref,
  tint = null,
  mine = false,
  dim = false,
  compact = false,
  label,
}: EmbedShellProps) {
  return (
    <div
      ref={ref}
      role="group"
      aria-label={label}
      className={`@container relative w-full min-w-0 overflow-hidden ${
        compact ? 'max-w-[320px] rounded-[14px] p-2' : 'max-w-[360px] rounded-[18px] p-3'
      }`}
      style={{
        background: background(dim ? null : tint, mine),
        border: '0.5px solid rgba(255,255,255,0.08)',
        boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.05), 0 8px 24px rgba(0,0,0,0.18)',
        filter: dim ? 'grayscale(1)' : undefined,
        opacity: dim ? 0.62 : undefined,
        transition: 'opacity 400ms var(--ease-apple)',
      }}
    >
      <span
        aria-hidden
        className="pointer-events-none absolute inset-x-5 top-0 h-px"
        style={{
          background: 'linear-gradient(90deg, transparent, rgba(255,255,255,0.16), transparent)',
        }}
      />
      {children}
    </div>
  );
}
