import type { TFunction } from 'i18next';
import { shortDate } from '../../../lib/chat/time';
import type { ChatError } from '../../../lib/chat/types';
import { notifyError } from '../../../lib/notify';

const KNOWN: ReadonlySet<string> = new Set([
  'muted',
  'slow_mode',
  'not_member',
  'send_paused',
  'send_limited',
  'request_limit',
  'busy',
  'offline',
  'unavailable',
  'disabled',
  'too_young',
  'not_found',
  'invalid',
]);

function asChatError(error: unknown): ChatError | null {
  if (typeof error !== 'object' || error === null || !('code' in error)) return null;
  return typeof error.code === 'string' ? (error as ChatError) : null;
}

export function errorText(error: unknown, t: TFunction, language: string): string {
  const chat = asChatError(error);
  if (!chat) return t('chat.thread.errors.generic');
  if (chat.code === 'muted' && chat.until) {
    return t('chat.thread.errors.mutedUntil', {
      time: shortDate(chat.until, Date.now(), language),
    });
  }
  if (chat.code === 'slow_mode' && chat.retryMs) {
    return t('chat.thread.errors.slowMode', { count: Math.ceil(chat.retryMs / 1000) });
  }
  return t(`chat.thread.errors.${KNOWN.has(chat.code) ? chat.code : 'generic'}`);
}

export function toastError(error: unknown, t: TFunction, language: string) {
  notifyError(errorText(error, t, language));
}
