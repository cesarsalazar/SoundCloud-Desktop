import type { ChatError } from '../chat/types';

const KNOWN = new Set([
  'muted',
  'slow_mode',
  'unavailable',
  'not_member',
  'not_found',
  'busy',
  'offline',
  'disabled',
  'invalid',
]);

export function chatError(error: unknown): ChatError | null {
  if (typeof error !== 'object' || error === null || !('code' in error)) return null;
  return typeof error.code === 'string' ? (error as ChatError) : null;
}

export function roomErrorKey(error: unknown): string {
  const code = chatError(error)?.code;
  return `chat.room.errors.${code && KNOWN.has(code) ? code : 'generic'}`;
}
