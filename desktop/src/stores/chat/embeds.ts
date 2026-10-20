import { create } from 'zustand';
import type { RoomPreview } from '../../lib/chat/types';

export interface ChatEmbedsState {
  roomPreviews: Record<string, RoomPreview>;
  setRoomPreview: (code: string, preview: RoomPreview) => void;
}

export const useChatEmbedsStore = create<ChatEmbedsState>()((set) => ({
  roomPreviews: {},
  setRoomPreview: (code, preview) =>
    set((state) => ({ roomPreviews: { ...state.roomPreviews, [code]: preview } })),
}));
