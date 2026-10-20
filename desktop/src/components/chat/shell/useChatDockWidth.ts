import { selectChatEnabled, useChatStore } from '../../../stores/chat';
import { useChatPrefsStore } from '../../../stores/chat/prefs';

export function useChatDockWidth(): number {
  const enabled = useChatStore(selectChatEnabled);
  const width = useChatPrefsStore((state) => (state.dockOpen ? state.dockWidth : 0));
  return enabled ? width : 0;
}
