import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import type { ConversationSummary, MessagePreview } from '../../../lib/chat/types';
import { Lock, Music } from '../../../lib/icons';
import { TypingDots } from './TypingDots';

interface RowPreviewProps {
  conversation: ConversationSummary;
  typing: boolean;
  mine: boolean;
}

function PreviewBody({ preview, mine }: { preview: MessagePreview; mine: boolean }) {
  const { t } = useTranslation();
  const you = mine ? <span className="shrink-0 text-white/55">{t('chat.list.you')}:</span> : null;
  switch (preview.kind) {
    case 'text':
      return (
        <>
          {you}
          <span className="truncate">{preview.text}</span>
        </>
      );
    case 'embed':
      return (
        <>
          {you}
          <Music size={12} className="shrink-0 text-[var(--color-accent)]" />
          <span className="truncate">{preview.text || t('chat.list.preview.embed')}</span>
        </>
      );
    case 'system':
      return <span className="truncate italic text-white/40">{preview.text}</span>;
    case 'deleted':
      return (
        <span className="truncate italic text-white/30">{t('chat.list.preview.deleted')}</span>
      );
    case 'locked':
      return (
        <>
          <Lock size={11} className="shrink-0 text-white/35" />
          <span className="truncate text-white/35">{t('chat.list.preview.locked')}</span>
        </>
      );
    default:
      return <span className="truncate text-white/35">{t('chat.list.preview.unknown')}</span>;
  }
}

export const RowPreview = memo(function RowPreview({
  conversation,
  typing,
  mine,
}: RowPreviewProps) {
  const { t } = useTranslation();
  if (typing) {
    return (
      <span className="flex min-w-0 items-center gap-1.5 text-[var(--color-accent)]">
        <TypingDots />
        <span className="truncate">{t('chat.list.typing')}</span>
      </span>
    );
  }
  return (
    <span className="flex min-w-0 items-center gap-1">
      {conversation.hasDraft && (
        <span className="shrink-0 font-semibold text-rose-300/90">{t('chat.list.draft')}</span>
      )}
      {conversation.preview && <PreviewBody preview={conversation.preview} mine={mine} />}
    </span>
  );
});
