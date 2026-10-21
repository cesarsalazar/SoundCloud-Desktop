import { useEffect } from 'react';
import { chatBridge } from '../../../lib/chat/bridge';
import type { ConversationSummary, ConvId } from '../../../lib/chat/types';
import { useChatStore } from '../../../stores/chat';
import { useChatListStore } from '../../../stores/chat/list';
import { useChatThreadStore } from '../../../stores/chat/thread';

export function useConversation(conversationId: ConvId): ConversationSummary | undefined {
  const summary = useChatListStore((state) => state.byId[conversationId]);
  const ready = useChatStore((state) => state.status?.phase === 'ready');
  const missing = summary === undefined;

  useEffect(() => {
    if (!missing || !ready) return;
    let live = true;
    chatBridge
      .open(conversationId)
      .then((snapshot) => {
        if (live) useChatListStore.getState().upsert(snapshot.conversation);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [conversationId, missing, ready]);

  return summary;
}

export function useThreadFocus(conversationId: ConvId) {
  useEffect(() => {
    useChatThreadStore.getState().setOpen(conversationId);
    void chatBridge.focus(conversationId).catch(() => undefined);
    return () => {
      if (useChatThreadStore.getState().openId !== conversationId) return;
      useChatThreadStore.getState().setOpen(null);
      void chatBridge.focus(null).catch(() => undefined);
    };
  }, [conversationId]);
}
