import { useEffect } from 'react';
import { chatBridge } from '../../../lib/chat/bridge';
import { initChatBridge, useChatStore } from '../../../stores/chat';
import { useChatListStore } from '../../../stores/chat/list';

function refetchConfig() {
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
      const phase = state.status?.phase;
      if (phase === prev.status?.phase) {
        if (state.status && state.status.requests !== prev.status?.requests) {
          useChatListStore.getState().setRequests(state.status.requests);
        }
        return;
      }
      if (state.config === null && phase && phase !== 'off') refetchConfig();
      if (phase === 'ready') void useChatListStore.getState().refresh();
    });
    return () => {
      stopped = true;
      stop?.();
      unsubscribe();
    };
  }, []);
}
