import type { Track } from '../../stores/player';
import type { Embed } from '../chat/types';

export function suggestion(track: Track): Embed {
  return {
    kind: 'track',
    urn: track.urn,
    permalink: track.permalink_url ?? '',
    secretToken: null,
    title: track.title,
    artist: track.user?.username ?? null,
    artworkUrl: track.artwork_url || track.user?.avatar_url || null,
    durationMs: track.duration || null,
    access: track.access ?? null,
    atMs: null,
    tracks: [],
    room: null,
  };
}
