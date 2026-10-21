const KNOWN = new Set(['too_young', 'not_found', 'offline', 'busy', 'device_revoked', 'disabled']);

export function deviceErrorKey(error: unknown): string {
  const code =
    typeof error === 'object' && error !== null && 'code' in error
      ? String((error as { code: unknown }).code)
      : '';
  return `chat.devices.errors.${KNOWN.has(code) ? code : 'generic'}`;
}
