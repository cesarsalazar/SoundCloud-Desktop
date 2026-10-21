import type { ChatErrorCode } from '../../../lib/chat/types';

const KNOWN: ReadonlySet<string> = new Set([
  'unavailable',
  'request_limit',
  'too_young',
  'send_limited',
  'busy',
  'offline',
  'disabled',
  'send_paused',
  'muted',
  'not_member',
  'slow_mode',
]);

export function chatErrorCode(error: unknown): ChatErrorCode | null {
  if (typeof error !== 'object' || error === null || !('code' in error)) return null;
  const code = (error as { code: unknown }).code;
  return typeof code === 'string' ? (code as ChatErrorCode) : null;
}

export function chatErrorKey(error: unknown): string {
  const code = chatErrorCode(error);
  return `chat.list.errors.${code && KNOWN.has(code) ? code : 'generic'}`;
}
