import * as Popover from '@radix-ui/react-popover';
import type { ReactNode } from 'react';
import { surface } from '../../../lib/chat/tone';
import { usePerfMode } from '../../../lib/perf';

interface RoomPopoverProps {
  trigger: ReactNode;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  width: number;
  children: ReactNode;
}

export function RoomPopover({ trigger, open, onOpenChange, width, children }: RoomPopoverProps) {
  const blur = usePerfMode().blur(28);
  return (
    <Popover.Root open={open} onOpenChange={onOpenChange}>
      <Popover.Trigger asChild>{trigger}</Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          side="bottom"
          align="start"
          sideOffset={8}
          collisionPadding={12}
          className="ctx-menu z-[300] rounded-[16px] p-1.5 outline-none"
          style={{
            width,
            background: blur > 0 ? surface(7, 86) : surface(8),
            backdropFilter: blur > 0 ? `blur(${blur}px) saturate(1.8)` : undefined,
            WebkitBackdropFilter: blur > 0 ? `blur(${blur}px) saturate(1.8)` : undefined,
            border: '1px solid var(--color-border-glass-hi)',
            boxShadow: '0 18px 50px rgba(0,0,0,0.55), inset 0 1px 0 rgba(255,255,255,0.05)',
          }}
        >
          {children}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
