import { api } from '../../../lib/api';
import { getCurrentTime } from '../../../lib/audio';
import type { Embed, EmbedTrack } from '../../../lib/chat/types';
import type { Playlist, SCUser } from '../../../lib/hooks';
import { kindOf } from '../../../lib/ids';
import { resolveLink } from '../../../lib/search';
import type { SoundCloudLink } from '../../../lib/soundcloudLink';
import { type Track, usePlayerStore } from '../../../stores/player';

const ARTWORK_HOST = /^https:\/\/([a-z0-9-]+\.)*sndcdn\.com\//i;
const TEXT_MAX = 160;
const PLAYLIST_TRACKS = 5;

function artwork(url: string | null | undefined): string | null {
  return url && ARTWORK_HOST.test(url) ? url : null;
}

function clip(text: string | null | undefined): string {
  const value = (text ?? '').trim();
  return value.length > TEXT_MAX ? `${value.slice(0, TEXT_MAX - 1)}…` : value;
}

function secretToken(permalink: string | undefined): string | null {
  if (!permalink) return null;
  try {
    return new URL(permalink).searchParams.get('secret_token');
  } catch {
    return null;
  }
}

function embedTrack(track: Track): EmbedTrack {
  return {
    urn: track.urn,
    title: clip(track.title),
    artist: clip(track.user.username),
    durationMs: track.duration,
    artworkUrl: artwork(track.artwork_url),
  };
}

export function trackEmbed(track: Track, atMs: number | null): Embed {
  return {
    kind: 'track',
    urn: track.urn,
    permalink: track.permalink_url ?? '',
    secretToken: secretToken(track.permalink_url),
    title: clip(track.title),
    artist: clip(track.user.username),
    artworkUrl: artwork(track.artwork_url ?? track.user.avatar_url),
    durationMs: track.duration,
    access: track.access ?? null,
    atMs,
    tracks: [],
    room: null,
  };
}

function playlistEmbed(playlist: Playlist): Embed {
  const tracks = playlist.tracks ?? [];
  return {
    kind: 'playlist',
    urn: playlist.urn,
    permalink: playlist.permalink_url ?? '',
    secretToken: secretToken(playlist.permalink_url),
    title: clip(playlist.title),
    artist: clip(playlist.user.username),
    artworkUrl: artwork(playlist.artwork_url ?? tracks[0]?.artwork_url),
    durationMs: playlist.duration,
    access: null,
    atMs: null,
    tracks: tracks.slice(0, PLAYLIST_TRACKS).map(embedTrack),
    room: null,
  };
}

function userEmbed(user: SCUser): Embed {
  return {
    kind: 'user',
    urn: user.urn,
    permalink: user.permalink_url ?? '',
    secretToken: null,
    title: clip(user.username),
    artist: null,
    artworkUrl: artwork(user.avatar_url),
    durationMs: null,
    access: null,
    atMs: null,
    tracks: [],
    room: null,
  };
}

export function nowPlayingEmbed(): Embed | null {
  const track = usePlayerStore.getState().currentTrack;
  return track ? trackEmbed(track, Math.round(getCurrentTime() * 1000)) : null;
}

export async function linkEmbed(link: SoundCloudLink): Promise<Embed | null> {
  const urn = link.kind === 'urn' ? link.urn : (await resolveLink(link.url)).urn;
  const path = encodeURIComponent(urn);
  const kind = kindOf(urn);
  if (kind === 'tracks') return trackEmbed(await api<Track>(`/tracks/${path}`), null);
  if (kind === 'playlists') return playlistEmbed(await api<Playlist>(`/playlists/${path}`));
  if (kind === 'users') return userEmbed(await api<SCUser>(`/users/${path}`));
  return null;
}
