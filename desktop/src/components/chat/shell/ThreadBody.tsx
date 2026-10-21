import { memo } from 'react';
import type { ConvId } from '../../../lib/chat/types';

export interface ThreadBodyProps {
  conversationId: ConvId;
  variant: 'page' | 'dock';
}

export const ThreadBody = memo(function ThreadBody({ conversationId }: ThreadBodyProps) {
  return <div className="relative min-h-0 flex-1" data-chat-thread={conversationId} />;
});
