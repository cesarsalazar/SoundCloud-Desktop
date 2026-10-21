import { type KeyboardEvent, memo } from 'react';
import { usePerfMode } from '../../../lib/perf';
import { useChatPrefsStore } from '../../../stores/chat/prefs';
import { DockBody } from './DockBody';
import { DockFrost } from './DockFrost';
import { DockResizeHandle } from './DockResizeHandle';
import { DockTabs } from './DockTabs';

export const DOCK_PLAYER_INSET = 136;
const OVERLAY_GAP = 48;

export const DockPanel = memo(function DockPanel({ mode }: { mode: 'docked' | 'overlay' }) {
  const perf = usePerfMode();
  const width = useChatPrefsStore((state) => state.dockWidth);
  const setDockOpen = useChatPrefsStore((state) => state.setDockOpen);
  const overlay = mode === 'overlay';

  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key !== 'Escape' || event.defaultPrevented) return;
    event.preventDefault();
    setDockOpen(false);
  };

  return (
    <>
      {overlay && (
        <div
          aria-hidden
          onClick={() => setDockOpen(false)}
          className="absolute inset-0 z-40 bg-black/45"
        />
      )}
      <aside
        data-ui="chat-dock"
        onKeyDown={onKeyDown}
        className={overlay ? 'absolute inset-y-0 right-0 z-50' : 'relative h-full shrink-0'}
        style={{
          width: overlay ? `min(${width}px, calc(100% - ${OVERLAY_GAP}px))` : width,
          boxShadow: overlay ? '-30px 0 80px rgba(0,0,0,0.45)' : undefined,
          animation:
            overlay && perf.mode !== 'light'
              ? 'chat-slide-in 320ms var(--ease-apple) both'
              : undefined,
        }}
      >
        <DockFrost />
        <DockResizeHandle />
        <div
          className="@container relative z-10 flex h-full min-h-0 flex-col"
          style={{ isolation: 'isolate', paddingBottom: DOCK_PLAYER_INSET }}
        >
          <DockTabs />
          <DockBody />
        </div>
      </aside>
    </>
  );
});
