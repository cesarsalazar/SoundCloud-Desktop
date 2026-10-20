import { create } from 'zustand';
import type { ConvId } from '../../lib/chat/types';

export interface ChatRoomState {
  code: string | null;
  conversationId: ConvId | null;
  setRoom: (code: string | null, conversationId: ConvId | null) => void;
}

export const useChatRoomStore = create<ChatRoomState>()((set) => ({
  code: null,
  conversationId: null,
  setRoom: (code, conversationId) => set({ code, conversationId }),
}));
