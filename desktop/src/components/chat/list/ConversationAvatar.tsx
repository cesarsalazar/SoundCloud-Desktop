import { memo, type ReactNode } from 'react';
import type { ConversationKind } from '../../../lib/chat/types';
import { Bookmark, Radio } from '../../../lib/icons';
import { Avatar } from '../../ui/Avatar';

interface ConversationAvatarProps {
  kind: ConversationKind;
  src: string | null;
  name: string;
  size: number;
}

function initials(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  return words
    .slice(0, 2)
    .map((word) => Array.from(word)[0] ?? '')
    .join('')
    .toUpperCase();
}

function GlyphTile({
  size,
  accent,
  children,
}: {
  size: number;
  accent?: boolean;
  children: ReactNode;
}) {
  return (
    <div
      className="flex shrink-0 items-center justify-center rounded-full"
      style={{
        width: size,
        height: size,
        color: accent ? 'var(--color-accent-contrast)' : 'rgba(255,255,255,0.75)',
        background: accent
          ? 'linear-gradient(145deg, var(--color-accent-hover), var(--color-accent))'
          : 'linear-gradient(145deg, rgba(255,255,255,0.14), rgba(255,255,255,0.04))',
        border: '0.5px solid rgba(255,255,255,0.12)',
      }}
    >
      {children}
    </div>
  );
}

export const ConversationAvatar = memo(function ConversationAvatar({
  kind,
  src,
  name,
  size,
}: ConversationAvatarProps) {
  const glyph = Math.round(size * 0.44);
  if (kind === 'self') {
    return (
      <GlyphTile size={size} accent>
        <Bookmark size={glyph} strokeWidth={2.2} />
      </GlyphTile>
    );
  }
  if (kind === 'room') {
    return (
      <GlyphTile size={size}>
        <Radio size={glyph} strokeWidth={2} />
      </GlyphTile>
    );
  }
  if (kind === 'direct') return <Avatar src={src} alt={name} size={size} />;
  if (src) {
    return (
      <img
        src={src}
        alt={name}
        loading="lazy"
        decoding="async"
        className="shrink-0 rounded-[32%] object-cover"
        style={{ width: size, height: size }}
      />
    );
  }
  return (
    <div
      className="flex shrink-0 items-center justify-center rounded-[32%] font-bold tracking-tight text-white/80"
      style={{
        width: size,
        height: size,
        fontSize: Math.round(size * 0.36),
        background:
          'linear-gradient(150deg, var(--color-accent-glow), rgba(255,255,255,0.05) 70%), rgba(255,255,255,0.04)',
        border: '0.5px solid rgba(255,255,255,0.1)',
      }}
    >
      {initials(name)}
    </div>
  );
});
