import { memo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Smile } from '../../../lib/chat/icons';
import { KeyCaps } from '../../ui/KeyCap';
import { emojiKeys } from './emoji-keys';

const HINT_MS = 2600;

export const EmojiButton = memo(function EmojiButton({
  onFocusInput,
}: {
  onFocusInput: () => void;
}) {
  const { t } = useTranslation();
  const [hint, setHint] = useState(false);

  useEffect(() => {
    if (!hint) return;
    const timer = setTimeout(() => setHint(false), HINT_MS);
    return () => clearTimeout(timer);
  }, [hint]);

  return (
    <div className="relative shrink-0">
      <button
        type="button"
        aria-label={t('chat.thread.composer.emoji')}
        onClick={() => {
          onFocusInput();
          setHint(true);
        }}
        className="flex size-9 cursor-pointer items-center justify-center rounded-full text-white/45 transition-colors duration-150 hover:bg-white/[0.06] hover:text-white/85"
      >
        <Smile size={19} />
      </button>
      {hint && (
        <div
          role="status"
          className="chat-thread-motion absolute right-0 bottom-full z-20 mb-2 flex items-center gap-2 whitespace-nowrap rounded-xl border border-white/[0.08] px-2.5 py-2 text-[12px] text-white/70 shadow-[0_12px_30px_rgba(0,0,0,0.5)]"
          style={{ background: 'rgb(26,26,31)', animation: 'chat-thread-rise 140ms ease-out' }}
        >
          <Smile size={14} className="text-white/45" />
          <KeyCaps labels={emojiKeys()} size="sm" />
        </div>
      )}
    </div>
  );
});
