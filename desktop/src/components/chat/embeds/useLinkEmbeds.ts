import { useQueries } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { findChatLinks } from '../../../lib/chat/soundcloud-link';
import type { Embed } from '../../../lib/chat/types';
import { snapshotFromLink } from './link-snapshot';

const SETTLE_MS = 350;

export interface PendingEmbed {
  key: string;
  embed: Embed | null;
  loading: boolean;
}

export interface LinkEmbeds {
  items: PendingEmbed[];
  embeds: Embed[];
  dismiss: (key: string) => void;
}

function useSettled(text: string): string {
  const [settled, setSettled] = useState(text);
  useEffect(() => {
    const timer = setTimeout(() => setSettled(text), SETTLE_MS);
    return () => clearTimeout(timer);
  }, [text]);
  return settled;
}

export function useLinkEmbeds(text: string, max?: number): LinkEmbeds {
  const settled = useSettled(text);
  const [dismissed, setDismissed] = useState<ReadonlySet<string>>(() => new Set());
  const links = useMemo(
    () => findChatLinks(settled, max).filter((link) => !dismissed.has(link.key)),
    [settled, max, dismissed],
  );
  const results = useQueries({
    queries: links.map((link) => ({
      queryKey: ['chat', 'link-embed', link.key, link.atMs],
      queryFn: () => snapshotFromLink(link),
      staleTime: Number.POSITIVE_INFINITY,
      retry: false,
    })),
  });
  const items = links.map((link, i) => ({
    key: link.key,
    embed: results[i]?.data ?? null,
    loading: results[i]?.isPending ?? false,
  }));
  const embeds = items.flatMap((item) => item.embed ?? []);
  const dismiss = useCallback(
    (key: string) => setDismissed((previous) => new Set(previous).add(key)),
    [],
  );
  return { items: items.filter((item) => item.loading || item.embed), embeds, dismiss };
}
