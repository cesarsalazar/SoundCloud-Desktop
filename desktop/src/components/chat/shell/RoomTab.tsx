import { memo } from 'react';
import type { ConvId } from '../../../lib/chat/types';
import { RoomDock } from '../room/RoomDock';
import { ThreadBody } from './ThreadBody';
import { useConversation, useThreadFocus } from './useConversation';

function RoomThread({ conversationId }: { conversationId: ConvId }) {
  useConversation(conversationId);
  useThreadFocus(conversationId);
  return <ThreadBody conversationId={conversationId} variant="dock" />;
}

export const RoomTab = memo(function RoomTab({ conversationId }: { conversationId: ConvId }) {
  return (
    <RoomDock>
      <RoomThread key={conversationId} conversationId={conversationId} />
    </RoomDock>
  );
});
