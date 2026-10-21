import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { Radio } from '../../../lib/icons';
import { usePerfMode } from '../../../lib/perf';
import { useChatListStore } from '../../../stores/chat/list';
import { useChatRoomStore } from '../../../stores/chat/room';
import { unreadLabel } from './mute';

export const RoomRow = memo(function RoomRow({ onOpen }: { onOpen: () => void }) {
  const { t } = useTranslation();
  const animate = usePerfMode().idleAnim;
  const code = useChatRoomStore((state) => state.code);
  const conversationId = useChatRoomStore((state) => state.conversationId);
  const summary = useChatListStore((state) =>
    conversationId ? state.byId[conversationId] : undefined,
  );
  if (!code || !conversationId) return null;
  const unread = summary?.unread ?? 0;

  return (
    <button
      type="button"
      onClick={onOpen}
      className="group mx-2 mb-1 flex h-[52px] w-[calc(100%-1rem)] cursor-pointer items-center gap-3 rounded-2xl border px-2.5 text-left transition-colors duration-200 hover:bg-white/[0.05]"
      style={{
        borderColor: 'var(--color-accent-glow)',
        background: 'linear-gradient(90deg, var(--color-accent-glow), transparent 70%)',
      }}
    >
      <span className="relative flex size-9 shrink-0 items-center justify-center rounded-full bg-[var(--color-accent)] text-[var(--color-accent-contrast)]">
        {animate && (
          <span
            aria-hidden
            className="chat-motion absolute inset-0 rounded-full border border-[var(--color-accent)]"
            style={{ animation: 'chat-live 1.8s ease-out infinite' }}
          />
        )}
        <Radio size={16} strokeWidth={2.2} />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-[13px] font-semibold text-white/90">
          {summary?.title ?? t('chat.list.room')}
        </span>
        <span className="block truncate font-mono text-[11px] tracking-wider text-white/40">
          {code}
        </span>
      </span>
      {unread > 0 && (
        <span className="rounded-full bg-[var(--color-accent)] px-1.5 text-[11px] font-bold leading-5 text-[var(--color-accent-contrast)] tabular-nums">
          {unreadLabel(unread)}
        </span>
      )}
    </button>
  );
});
