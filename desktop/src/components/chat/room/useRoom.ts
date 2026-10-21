import { useEffect, useMemo, useState } from 'react';
import type { ConvId, Member } from '../../../lib/chat/types';
import { serverTime } from '../../../lib/together/link';
import { canModerate, chatMembers, isHost, mutedUntil } from '../../../lib/together/room-rules';
import type { RoomView } from '../../../lib/together/types';
import { useChatRoomStore } from '../../../stores/chat/room';
import { useTogetherStore } from '../../../stores/together';

export interface RoomSelf {
  id: string | null;
  host: boolean;
  moderates: boolean;
}

export function useRoom(): RoomView | null {
  return useTogetherStore((state) => state.room);
}

export function useRoomSelf(room: RoomView | null): RoomSelf {
  const id = useTogetherStore((state) => state.selfId);
  const host = !!room && isHost(room, id);
  const moderates = !!room && canModerate(room, id);
  return useMemo(() => ({ id, host, moderates }), [id, host, moderates]);
}

export function useServerNow(stepMs: number, active: boolean): number {
  const [now, setNow] = useState(serverTime);
  useEffect(() => {
    if (!active) return;
    setNow(serverTime());
    const timer = setInterval(() => setNow(serverTime()), stepMs);
    return () => clearInterval(timer);
  }, [stepMs, active]);
  return now;
}

export function useMutedUntil(room: RoomView | null, userId: string | null): number | null {
  const pending = room?.muted?.some((entry) => entry.userId === userId) ?? false;
  const now = useServerNow(1_000, pending);
  return room ? mutedUntil(room, userId, now) : null;
}

export function useRoomSendLock(conversationId: ConvId): number | null {
  const ours = useChatRoomStore((state) => state.conversationId === conversationId);
  const room = useRoom();
  const self = useTogetherStore((state) => state.selfId);
  return useMutedUntil(ours ? room : null, self);
}

export function useRoomMembers(conversationId: ConvId): Member[] | undefined {
  const ours = useChatRoomStore((state) => state.conversationId === conversationId);
  const room = useRoom();
  const view = ours ? room : null;
  return useMemo(() => (view ? chatMembers(view) : undefined), [view]);
}
