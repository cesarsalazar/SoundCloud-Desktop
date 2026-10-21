import { memo, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { BellOff, Timer } from '../../../lib/chat/icons';
import type { ConversationSummary, ConvId } from '../../../lib/chat/types';
import { ChevronLeft, Fullscreen, PictureInPicture2, X } from '../../../lib/icons';
import { isRequest } from '../../../stores/chat/list';
import { useChatPrefsStore } from '../../../stores/chat/prefs';
import { Skeleton } from '../../ui/Skeleton';
import { ConversationAvatar } from '../list/ConversationAvatar';
import { conversationAvatar, conversationTitle } from '../list/conversation-meta';
import { isMuted } from '../list/mute';
import { lastOutsidePath } from './useDockMode';
import { chatPath } from './useOpenChat';

interface ThreadChromeProps {
  conversationId: ConvId;
  conversation: ConversationSummary | undefined;
  variant: 'page' | 'dock';
  onBack?: () => void;
}

function ChromeButton({
  label,
  onClick,
  className,
  children,
}: {
  label: string;
  onClick: () => void;
  className?: string;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={onClick}
      className={`flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-full text-white/45 transition-colors hover:bg-white/[0.08] hover:text-white/90 ${className ?? ''}`}
    >
      {children}
    </button>
  );
}

function Subtitle({ conversation }: { conversation: ConversationSummary }) {
  const { t } = useTranslation();
  const parts: string[] = [];
  if (isRequest(conversation)) parts.push(t('chat.list.requests.badge'));
  if (conversation.kind === 'group') {
    parts.push(t('chat.list.members', { count: conversation.memberCount }));
  }
  if (conversation.kind === 'self') parts.push(t('chat.list.savedHint'));
  if (conversation.kind === 'room') parts.push(conversation.roomCode ?? t('chat.list.room'));
  if (conversation.ttlS !== null) parts.push(t(`chat.list.ttl.${conversation.ttlS}`));
  const muted = isMuted(conversation, Date.now());
  if (parts.length === 0 && !muted) return null;
  return (
    <p className="flex items-center gap-1.5 truncate text-[11.5px] text-white/40">
      {conversation.ttlS !== null && <Timer size={11} className="shrink-0" />}
      {muted && <BellOff size={11} className="shrink-0" />}
      <span className="truncate">{parts.join(' · ')}</span>
    </p>
  );
}

export const ThreadChrome = memo(function ThreadChrome({
  conversationId,
  conversation,
  variant,
  onBack,
}: ThreadChromeProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const title = conversation ? conversationTitle(conversation, t) : '';

  const toDock = () => {
    useChatPrefsStore.getState().openInDock(conversationId);
    navigate(lastOutsidePath());
  };

  return (
    <div className="flex h-14 shrink-0 items-center gap-2.5 border-b border-white/[0.05] px-3">
      {onBack && (
        <ChromeButton
          label={t('chat.list.dock.back')}
          onClick={onBack}
          className="@min-[641px]:hidden"
        >
          <ChevronLeft size={18} />
        </ChromeButton>
      )}
      {conversation ? (
        <ConversationAvatar
          kind={conversation.kind}
          src={conversationAvatar(conversation)}
          name={title}
          size={34}
        />
      ) : (
        <Skeleton rounded="full" className="size-[34px] shrink-0" />
      )}
      <div className="min-w-0 flex-1">
        {conversation ? (
          <p className="truncate text-[14px] font-bold tracking-tight text-white/92">{title}</p>
        ) : (
          <Skeleton className="h-3 w-32" />
        )}
        {conversation && <Subtitle conversation={conversation} />}
      </div>
      {variant === 'page' ? (
        <ChromeButton label={t('chat.list.dock.toDock')} onClick={toDock}>
          <PictureInPicture2 size={16} />
        </ChromeButton>
      ) : (
        <>
          <ChromeButton
            label={t('chat.list.dock.expand')}
            onClick={() => navigate(chatPath(conversationId))}
          >
            <Fullscreen size={15} />
          </ChromeButton>
          {conversation?.kind !== 'room' && (
            <ChromeButton
              label={t('chat.list.dock.closeTab')}
              onClick={() => useChatPrefsStore.getState().closeTab(conversationId)}
            >
              <X size={15} />
            </ChromeButton>
          )}
        </>
      )}
    </div>
  );
});
