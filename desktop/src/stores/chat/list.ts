import { create } from 'zustand';
import type { ConversationSummary, GoneEvent, ListFilter, SyncEvent } from '../../lib/chat/types';

export interface ChatListState {
  filter: ListFilter;
  items: ConversationSummary[];
  nextCursor: string | null;
  requests: number;
  setFilter: (filter: ListFilter) => void;
  applySync: (event: SyncEvent) => void;
  applyGone: (event: GoneEvent) => void;
}

export const useChatListStore = create<ChatListState>()((set) => ({
  filter: 'all',
  items: [],
  nextCursor: null,
  requests: 0,
  setFilter: (filter) => set({ filter }),
  applySync: () => undefined,
  applyGone: () => undefined,
}));
