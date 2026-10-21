import { useEffect } from 'react';
import { chatBridge } from '../../../lib/chat/bridge';
import { initChatBridge, useChatStore } from '../../../stores/chat';
import { useChatListStore } from '../../../stores/chat/list';

function refetchConfig() {
  if (useChatStore.getState().config !== null) return;
  chatBridge
    .config()
    .then((config) => useChatStore.getState().setConfig(config))
    .catch(() => undefined);
}

export function useChatBoot() {
  useEffect(() => {
    let stopped = false;
    let stop: (() => void) | null = null;
    void initChatBridge()
      .then((unlisten) => {
        if (stopped) unlisten();
        else stop = unlisten;
      })
      .catch(() => undefined);
    const unsubscribe = useChatStore.subscribe((state, prev) => {
      const status = state.status;
      if (status && status.requests !== prev.status?.requests) {
        useChatListStore.getState().setRequests(status.requests);
      }
      const phase = status?.phase;
      if (phase === prev.status?.phase) return;
      if (state.config === null && phase && phase !== 'off') queueMicrotask(refetchConfig);
      if (phase === 'ready') void useChatListStore.getState().refresh();
    });
    return () => {
      stopped = true;
      stop?.();
      unsubscribe();
    };
  }, []);
}
