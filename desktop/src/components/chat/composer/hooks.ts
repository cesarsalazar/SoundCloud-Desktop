import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { chatBridge } from '../../../lib/chat/bridge';
import { composeRich, type MentionPick } from '../../../lib/chat/spans';
import type { ConvId, Embed } from '../../../lib/chat/types';
import { findSoundCloudLink } from '../../../lib/soundcloudLink';
import { linkEmbed } from './embed-source';

const DRAFT_MS = 800;
const TYPING_MS = 4_000;
const PREVIEW_MS = 350;

function saveDraft(conversationId: ConvId, text: string, picks: MentionPick[]) {
  const rich = text.trim() ? composeRich(text, picks) : null;
  void chatBridge.draftSave(conversationId, rich).catch(() => undefined);
}

export function useDraftSaver(
  conversationId: ConvId,
  text: string,
  picks: MentionPick[],
  enabled: boolean,
) {
  const saved = useRef(text);
  const pending = useRef<{ text: string; picks: MentionPick[] } | null>(null);

  useEffect(() => {
    if (!enabled || text === saved.current) {
      pending.current = null;
      return;
    }
    pending.current = { text, picks };
    const timer = setTimeout(() => {
      saved.current = text;
      pending.current = null;
      saveDraft(conversationId, text, picks);
    }, DRAFT_MS);
    return () => clearTimeout(timer);
  }, [conversationId, text, picks, enabled]);

  useEffect(
    () => () => {
      if (pending.current) saveDraft(conversationId, pending.current.text, pending.current.picks);
    },
    [conversationId],
  );

  return useCallback((value: string) => {
    saved.current = value;
  }, []);
}

export function useTypingPing(conversationId: ConvId) {
  const last = useRef(0);
  return useCallback(() => {
    const now = Date.now();
    if (now - last.current < TYPING_MS) return;
    last.current = now;
    void chatBridge.typing(conversationId).catch(() => undefined);
  }, [conversationId]);
}

interface Preview {
  id: string;
  embed: Embed | null;
}

export function useLinkPreview(text: string) {
  const link = useMemo(() => findSoundCloudLink(text), [text]);
  const id = link ? (link.kind === 'urn' ? link.urn : link.url) : null;
  const [dismissed, setDismissed] = useState<string | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);

  useEffect(() => {
    if (!link || !id || id === dismissed || preview?.id === id) return;
    let alive = true;
    const timer = setTimeout(() => {
      linkEmbed(link)
        .then((embed) => alive && setPreview({ id, embed }))
        .catch(() => alive && setPreview({ id, embed: null }));
    }, PREVIEW_MS);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [link, id, dismissed, preview?.id]);

  const active = id !== null && id !== dismissed;
  return {
    embed: active && preview?.id === id ? preview.embed : null,
    loading: active && preview?.id !== id,
    dismiss: () => setDismissed(id),
  };
}
