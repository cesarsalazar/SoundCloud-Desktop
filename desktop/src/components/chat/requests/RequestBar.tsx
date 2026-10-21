import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { UserPlus } from '../../../lib/chat/icons';
import type { ConversationSummary } from '../../../lib/chat/types';
import { useChatListStore } from '../../../stores/chat/list';
import { RequestActions } from './RequestActions';

export const RequestBar = memo(function RequestBar({
  conversation,
}: {
  conversation: ConversationSummary;
}) {
  const { t } = useTranslation();
  const request = useChatListStore((state) =>
    state.requestItems.find((item) => item.conversation.id === conversation.id),
  );
  const from = request?.from ?? conversation.peer;
  if (!from) return null;
  const group = conversation.kind === 'group';

  return (
    <div
      className="mx-3 mt-2 flex flex-wrap items-center gap-x-3 gap-y-2 rounded-2xl border px-3 py-2"
      style={{
        borderColor: 'var(--color-accent-glow)',
        background: 'linear-gradient(90deg, var(--color-accent-glow), rgba(255,255,255,0.025) 80%)',
      }}
    >
      <UserPlus size={16} className="shrink-0 text-[var(--color-accent)]" />
      <p className="min-w-[8rem] flex-1 truncate text-[12.5px] font-medium text-white/80">
        {group
          ? t('chat.list.requests.invitedBy', { name: from.name })
          : t('chat.list.requests.wantsToWrite', { name: from.name })}
      </p>
      <RequestActions conversation={conversation} from={from} wide />
    </div>
  );
});
