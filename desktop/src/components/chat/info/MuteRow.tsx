import { memo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { BellOff } from '../../../lib/chat/icons';
import { Bell } from '../../../lib/icons';
import { useThread } from '../thread/context';
import { toastError } from '../thread/errors';
import { isMuted, setMuted } from '../thread/thread-menu';

export const MuteRow = memo(function MuteRow() {
  const { t, i18n } = useTranslation();
  const { conversation } = useThread();
  const [busy, setBusy] = useState(false);
  const muted = isMuted(conversation, Date.now());
  const Icon = muted ? BellOff : Bell;

  const toggle = async () => {
    setBusy(true);
    try {
      await setMuted(conversation, !muted);
    } catch (error) {
      toastError(error, t, i18n.language);
    } finally {
      setBusy(false);
    }
  };

  return (
    <button
      type="button"
      role="switch"
      aria-checked={!muted}
      disabled={busy}
      onClick={() => void toggle()}
      className="flex w-full cursor-pointer items-center gap-2.5 rounded-2xl border border-white/[0.06] bg-white/[0.025] px-3 py-2.5 text-left transition-colors duration-150 hover:bg-white/[0.045] disabled:opacity-60"
    >
      <span className="flex size-7 items-center justify-center rounded-full bg-white/[0.06] text-white/60">
        <Icon size={14} />
      </span>
      <span className="flex-1 text-[13px] font-semibold text-white/85">
        {t('chat.thread.info.notifications')}
      </span>
      <span
        aria-hidden
        className="relative h-5 w-9 rounded-full transition-colors duration-200"
        style={{ background: muted ? 'rgba(255,255,255,0.12)' : 'var(--color-accent)' }}
      >
        <span
          className="absolute top-0.5 left-0.5 size-4 rounded-full bg-white shadow transition-transform duration-200"
          style={{ transform: muted ? 'translateX(0)' : 'translateX(16px)' }}
        />
      </span>
    </button>
  );
});
