import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import type { Embed } from '../../../lib/chat/types';
import { art, formatTime } from '../../../lib/formatters';
import { ListMusic, Loader2, Music, Play, User, X } from '../../../lib/icons';

const KIND_ICONS = { track: Music, playlist: ListMusic, user: User, room_invite: Music } as const;

function EmbedChip({ embed, onRemove }: { embed: Embed; onRemove: () => void }) {
  const { t } = useTranslation();
  const Icon = KIND_ICONS[embed.kind];
  const cover = art(embed.artworkUrl, 't67x67');
  return (
    <div className="flex min-w-0 max-w-full items-center gap-2.5 rounded-2xl border border-white/[0.06] bg-white/[0.04] py-1.5 pr-1.5 pl-1.5">
      <span
        className={`relative flex size-9 shrink-0 items-center justify-center overflow-hidden bg-white/[0.06] text-white/45 ${
          embed.kind === 'user' ? 'rounded-full' : 'rounded-[9px]'
        }`}
      >
        {cover ? <img src={cover} alt="" className="size-full object-cover" /> : <Icon size={15} />}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-[12.5px] font-semibold text-white/88">
          {embed.title}
        </span>
        <span className="flex min-w-0 items-center gap-1.5 text-[11.5px] text-white/45">
          {embed.atMs !== null && (
            <span
              className="inline-flex shrink-0 items-center gap-[3px] rounded-md px-1.5 font-semibold tabular-nums"
              style={{ color: 'var(--color-accent-hover)', background: 'var(--color-accent-glow)' }}
            >
              <Play size={7} fill="currentColor" />
              {formatTime(embed.atMs / 1000)}
            </span>
          )}
          {embed.artist && <span className="truncate">{embed.artist}</span>}
        </span>
      </span>
      <button
        type="button"
        aria-label={t('chat.thread.composer.removeAttachment')}
        onClick={onRemove}
        className="flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-full text-white/40 transition-colors duration-150 hover:bg-white/[0.08] hover:text-white"
      >
        <X size={14} />
      </button>
    </div>
  );
}

export const AttachedEmbeds = memo(function AttachedEmbeds({
  embeds,
  loading,
  onRemove,
}: {
  embeds: Embed[];
  loading: boolean;
  onRemove: (index: number) => void;
}) {
  if (embeds.length === 0 && !loading) return null;
  return (
    <div className="mx-1 mb-1.5 flex flex-col gap-1">
      {embeds.map((embed, index) => (
        <EmbedChip key={`${embed.urn}:${index}`} embed={embed} onRemove={() => onRemove(index)} />
      ))}
      {loading && (
        <div className="flex h-12 items-center gap-2 rounded-2xl bg-white/[0.03] px-3 text-white/40">
          <Loader2 size={14} className="animate-spin" />
          <span className="h-2.5 w-32 rounded-full bg-white/[0.06]" />
        </div>
      )}
    </div>
  );
});
