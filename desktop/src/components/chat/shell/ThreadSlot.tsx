import { memo } from 'react';
import type { ConvId } from '../../../lib/chat/types';
import { isRequest } from '../../../stores/chat/list';
import { RequestBar } from '../requests/RequestBar';
import { ThreadBody } from './ThreadBody';
import { ThreadChrome } from './ThreadChrome';
import { useConversation, useThreadFocus } from './useConversation';

interface ThreadSlotProps {
  conversationId: ConvId;
  variant: 'page' | 'dock';
  onBack?: () => void;
}

export const ThreadSlot = memo(function ThreadSlot({
  conversationId,
  variant,
  onBack,
}: ThreadSlotProps) {
  const conversation = useConversation(conversationId);
  useThreadFocus(conversationId);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <ThreadChrome
        conversationId={conversationId}
        conversation={conversation}
        variant={variant}
        onBack={onBack}
      />
      {conversation && isRequest(conversation) && <RequestBar conversation={conversation} />}
      <ThreadBody conversationId={conversationId} variant={variant} />
    </div>
  );
});
