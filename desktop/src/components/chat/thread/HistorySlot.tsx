import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { Lock } from '../../../lib/chat/icons';
import { timeLeft } from '../../../lib/chat/time';
import { useChatStore } from '../../../stores/chat';
import { useMinuteNow } from '../bubbles/useMinuteNow';
import { useThread } from './context';

export const HistorySlot = memo(function HistorySlot() {
  const { t, i18n } = useTranslation();
  const { conversation } = useThread();
  const gate = useChatStore((state) => state.status?.history ?? null);
  const now = useMinuteNow();
  const locked = conversation.historyLocked || gate?.locked === true;
  if (!locked) return null;
  const unlocksAt = gate?.unlocksAt ?? null;

  return (
    <div className="flex shrink-0 items-center gap-2.5 border-b border-white/[0.05] bg-white/[0.025] px-3.5 py-2">
      <span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-white/[0.06] text-white/60">
        <Lock size={13} />
      </span>
      <span className="min-w-0 flex-1 text-[12px] leading-snug text-white/60">
        {t('chat.thread.history.locked')}
      </span>
      {unlocksAt !== null && unlocksAt > now && (
        <span className="shrink-0 rounded-full bg-white/[0.06] px-2 py-0.5 text-[11px] font-semibold tabular-nums text-white/55">
          {timeLeft(unlocksAt, now, i18n.language)}
        </span>
      )}
    </div>
  );
});
