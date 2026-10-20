import { create } from 'zustand';
import type {
  ConversationMarks,
  ConvId,
  GoneEvent,
  Message,
  MessagesEvent,
  PeerMark,
  Seq,
  TypingEvent,
  UserId,
} from '../../lib/chat/types';

export interface ThreadState {
  messages: Message[];
  moreBefore: boolean;
  marks: PeerMark[];
  pins: Seq[];
  typing: UserId[];
}

export interface ChatThreadState {
  openId: ConvId | null;
  threads: Record<ConvId, ThreadState>;
  setOpen: (conversationId: ConvId | null) => void;
  applyMessages: (event: MessagesEvent) => void;
  applyTyping: (event: TypingEvent) => void;
  applyMarks: (marks: ConversationMarks[]) => void;
  applyGone: (event: GoneEvent) => void;
}

export const useChatThreadStore = create<ChatThreadState>()((set) => ({
  openId: null,
  threads: {},
  setOpen: (openId) => set({ openId }),
  applyMessages: () => undefined,
  applyTyping: () => undefined,
  applyMarks: () => undefined,
  applyGone: () => undefined,
}));
