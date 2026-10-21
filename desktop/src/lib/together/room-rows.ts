import type { ConversationKind, SystemEvent } from '../chat/types';

export type RoomRowKind = 'track' | 'suggest' | 'moderator';

export function roomRowKind(event: SystemEvent): RoomRowKind | null {
  if (event.kind === 'track' && event.track) return 'track';
  if (event.kind !== 'other' || !event.actor) return null;
  if (event.track) return 'suggest';
  return event.target ? 'moderator' : null;
}

export function canForwardFrom(kind: ConversationKind): boolean {
  return kind !== 'room';
}
