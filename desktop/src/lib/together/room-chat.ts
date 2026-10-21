import { type ChatState, selectChatEnabled, useChatStore } from '../../stores/chat';
import { useChatPrefsStore } from '../../stores/chat/prefs';
import { useChatRoomStore } from '../../stores/chat/room';
import { useTogetherStore } from '../../stores/together';
import { chatBridge } from '../chat/bridge';
import { onChatEvent } from '../chat/events';
import { chatError } from './chat-errors';
import {
  gone,
  moved,
  NO_ROOM_CHAT,
  nextStep,
  opened,
  opening,
  type RoomChatWish,
  refused,
  retry,
} from './room-chat-state';

const RETRY_MS = 15_000;
const NOT_MEMBER_RETRY_MS = 1_500;

let retryTimer: ReturnType<typeof setTimeout> | null = null;

function chatReady(state: ChatState): boolean {
  return selectChatEnabled(state) && state.status?.phase === 'ready';
}

function wish(): RoomChatWish | null {
  const { room, phase } = useTogetherStore.getState();
  if (!room || phase === 'idle' || phase === 'connecting') return null;
  if (room.chatEnabled !== true || !chatReady(useChatStore.getState())) return null;
  return { code: room.code, chatId: room.chatId ?? null };
}

function later(ms: number): void {
  if (retryTimer) clearTimeout(retryTimer);
  retryTimer = setTimeout(() => {
    retryTimer = null;
    useChatRoomStore.getState().apply(retry);
    sync();
  }, ms);
}

async function open(code: string): Promise<void> {
  useChatRoomStore.getState().apply(() => opening(code));
  try {
    const summary = await chatBridge.roomChat(code);
    useChatRoomStore.getState().apply((state) => opened(state, code, summary.id));
  } catch (error) {
    const failure = chatError(error);
    const off = failure?.code === 'unavailable';
    useChatRoomStore.getState().apply((state) => refused(state, code, off));
    if (off) return;
    if (failure?.code === 'not_member') later(NOT_MEMBER_RETRY_MS);
    else later(failure?.retryMs ?? RETRY_MS);
  }
}

function sync(): void {
  const room = useChatRoomStore.getState();
  const step = nextStep(room, wish());
  if (step.kind === 'open') void open(step.code);
  if (step.kind === 'move') room.apply((state) => moved(state, step.code));
  if (step.kind === 'clear') room.apply(() => NO_ROOM_CHAT);
}

export function openRoomDock(): void {
  useChatPrefsStore.getState().setDockOpen(true);
}

export function retryRoomChat(): void {
  useChatRoomStore.getState().apply(retry);
  sync();
}

export function startRoomChat(): void {
  useTogetherStore.subscribe((state, previous) => {
    if (state.room !== previous.room || state.phase !== previous.phase) sync();
  });
  useChatStore.subscribe((state, previous) => {
    if (chatReady(state) === chatReady(previous)) return;
    useChatRoomStore.getState().apply(retry);
    sync();
  });
  void onChatEvent('chat:gone', (event) =>
    useChatRoomStore.getState().apply((state) => gone(state, event.conversations)),
  );
  sync();
}
