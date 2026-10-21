import { memo, type ReactNode, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Archive } from '../../../lib/chat/icons';
import type { ListFilter } from '../../../lib/chat/types';
import { useChatStore } from '../../../stores/chat';
import { useChatListStore } from '../../../stores/chat/list';
import { unreadLabel } from './mute';

const FILTERS: ListFilter[] = ['all', 'direct', 'groups', 'unread', 'requests'];

const ACTIVE = {
  background:
    'linear-gradient(180deg, var(--color-accent-glow), transparent), rgba(255,255,255,0.07)',
  borderColor: 'var(--color-accent-glow)',
  color: '#fff',
};

function Chip({
  active,
  title,
  onClick,
  children,
}: {
  active: boolean;
  title?: string;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      title={title}
      aria-pressed={active}
      onClick={onClick}
      className={`flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-3 text-[12px] font-semibold whitespace-nowrap transition-colors duration-200 ${
        active
          ? ''
          : 'border-white/[0.06] bg-white/[0.03] text-white/50 hover:bg-white/[0.07] hover:text-white/80'
      }`}
      style={active ? ACTIVE : undefined}
    >
      {children}
    </button>
  );
}

function Count({ value, loud }: { value: number; loud: boolean }) {
  if (value <= 0) return null;
  return (
    <span
      className={`rounded-full px-1.5 text-[10.5px] font-bold tabular-nums leading-[16px] ${
        loud ? 'bg-[var(--color-accent)] text-[var(--color-accent-contrast)]' : 'bg-white/10'
      }`}
    >
      {unreadLabel(value)}
    </span>
  );
}

export const FilterChips = memo(function FilterChips() {
  const { t } = useTranslation();
  const filter = useChatListStore((state) => state.filter);
  const setFilter = useChatListStore((state) => state.setFilter);
  const requests = useChatListStore((state) => state.requests);
  const unread = useChatStore((state) => state.status?.unread ?? 0);
  const rowRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const row = rowRef.current;
    if (!row) return;
    const onWheel = (event: WheelEvent) => {
      if (Math.abs(event.deltaY) <= Math.abs(event.deltaX)) return;
      if (row.scrollWidth <= row.clientWidth) return;
      event.preventDefault();
      row.scrollLeft += event.deltaY;
    };
    row.addEventListener('wheel', onWheel, { passive: false });
    return () => row.removeEventListener('wheel', onWheel);
  }, []);

  const counts: Partial<Record<ListFilter, number>> = { unread, requests };

  return (
    <div
      ref={rowRef}
      className="scrollbar-hide flex items-center gap-1.5 overflow-x-auto px-3 pb-2"
    >
      {FILTERS.map((id) => (
        <Chip key={id} active={filter === id} onClick={() => setFilter(id)}>
          {t(`chat.list.filters.${id}`)}
          <Count value={counts[id] ?? 0} loud={id === 'requests' && filter !== id} />
        </Chip>
      ))}
      <Chip
        active={filter === 'archived'}
        title={t('chat.list.filters.archived')}
        onClick={() => setFilter(filter === 'archived' ? 'all' : 'archived')}
      >
        <Archive size={13} />
      </Chip>
    </div>
  );
});
