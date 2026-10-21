import { memo } from 'react';
import type { Embed } from '../../../lib/chat/types';
import { PlaylistEmbed } from './PlaylistEmbed';
import { RoomInviteEmbed } from './RoomInviteEmbed';
import { TrackEmbed } from './TrackEmbed';
import { UserEmbed } from './UserEmbed';

export interface EmbedCardProps {
  embed: Embed;
  mine?: boolean;
  compact?: boolean;
}

export const EmbedCard = memo(function EmbedCard({
  embed,
  mine = false,
  compact = false,
}: EmbedCardProps) {
  switch (embed.kind) {
    case 'track':
      return <TrackEmbed embed={embed} mine={mine} compact={compact} />;
    case 'playlist':
      return <PlaylistEmbed embed={embed} mine={mine} compact={compact} />;
    case 'user':
      return <UserEmbed embed={embed} mine={mine} compact={compact} />;
    case 'room_invite':
      return <RoomInviteEmbed embed={embed} mine={mine} compact={compact} />;
    default:
      return null;
  }
});
