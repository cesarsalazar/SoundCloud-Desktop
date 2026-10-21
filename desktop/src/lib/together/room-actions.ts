import { toast } from 'sonner';
import i18n from '../../i18n';
import { type Track, usePlayerStore } from '../../stores/player';
import { useTogetherStore } from '../../stores/together';
import { api } from '../api';
import { chatBridge } from '../chat/bridge';
import { trackEmbed } from '../chat/embed-snapshot';
import { chatErrorKey } from '../chat/errors';
import type { RoomModeration, RoomSettingsPatch } from '../chat/types';
import { cleanTitle } from './room-rules';

const ROOM_ERRORS: ReadonlySet<string> = new Set([
  'muted',
  'slow_mode',
  'unavailable',
  'not_member',
  'not_found',
  'busy',
  'offline',
  'disabled',
  'invalid',
]);

async function inRoom(task: (code: string) => Promise<void>): Promise<boolean> {
  const code = useTogetherStore.getState().room?.code;
  if (!code) return false;
  try {
    await task(code);
    return true;
  } catch (error) {
    toast.error(i18n.t(chatErrorKey(error, 'room', ROOM_ERRORS)));
    return false;
  }
}

function moderate(action: RoomModeration): Promise<boolean> {
  return inRoom((code) => chatBridge.roomModerate(code, action));
}

function configure(patch: RoomSettingsPatch): Promise<boolean> {
  return inRoom((code) => chatBridge.roomSettings(code, patch));
}

export function kickMember(userId: string): Promise<boolean> {
  return moderate({ action: 'kick', userId });
}

export function banMember(userId: string): Promise<boolean> {
  return moderate({ action: 'ban', userId });
}

export function muteMember(userId: string, minutes: number): Promise<boolean> {
  return moderate({ action: 'mute', userId, seconds: minutes * 60 });
}

export function unmuteMember(userId: string): Promise<boolean> {
  return moderate({ action: 'mute', userId, seconds: 0 });
}

export function setModerator(userId: string, on: boolean): Promise<boolean> {
  return moderate({ action: 'moderator', userId, on });
}

export function renameRoom(title: string): Promise<boolean> {
  return configure({ title: cleanTitle(title) });
}

export function setRoomChat(on: boolean): Promise<boolean> {
  return configure({ chatEnabled: on });
}

export function setRoomLinks(on: boolean): Promise<boolean> {
  return configure({ links: on });
}

export function setSlowMode(seconds: number): Promise<boolean> {
  return configure({ slowModeS: seconds });
}

export async function suggestTrack(track: Track): Promise<boolean> {
  const embed = trackEmbed(track);
  if (embed) return inRoom((code) => chatBridge.roomSuggest(code, embed));
  toast.error(i18n.t('chat.room.errors.invalid'));
  return false;
}

export async function queueSuggestion(urn: string): Promise<boolean> {
  try {
    const track = await api<Track>(`/tracks/${encodeURIComponent(urn)}`);
    usePlayerStore.getState().addToQueue([track]);
    return true;
  } catch {
    toast.error(i18n.t('chat.room.errors.generic'));
    return false;
  }
}
