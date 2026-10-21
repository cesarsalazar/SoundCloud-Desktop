import { memo } from 'react';
import { art } from '../../../lib/formatters';
import { usePerfMode } from '../../../lib/perf';
import type { RoomView } from '../../../lib/together/types';

const LAYER = { contain: 'strict', transform: 'translateZ(0)' } as const;

export const RoomSleeve = memo(function RoomSleeve({ room }: { room: RoomView }) {
  const perf = usePerfMode();
  const track = room.playback.track;
  const cover = track ? art(track.artwork_url || track.user?.avatar_url, 't200x200') : null;
  const blur = perf.blur(34);
  const washed = perf.bloom && blur > 0 && cover;

  return (
    <div aria-hidden className="pointer-events-none absolute inset-0 overflow-hidden" style={LAYER}>
      {washed && (
        <img
          key={cover}
          src={cover}
          alt=""
          className="absolute -inset-8 h-[calc(100%+4rem)] w-[calc(100%+4rem)] max-w-none object-cover opacity-50 animate-[room-fade_0.8s_var(--ease-apple)_both]"
          style={{ filter: `blur(${blur}px) saturate(1.5)` }}
        />
      )}
      <div
        className="absolute inset-0"
        style={{ background: 'linear-gradient(160deg, var(--color-accent-glow), transparent 70%)' }}
      />
      <div className="absolute inset-0 bg-gradient-to-b from-black/5 via-black/25 to-chat-cutout" />
    </div>
  );
});
