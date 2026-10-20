import { memo } from 'react';
import type { Embed } from '../../../lib/chat/types';

export interface EmbedCardProps {
  embed: Embed;
  mine?: boolean;
  compact?: boolean;
}

export const EmbedCard = memo(function EmbedCard(_props: EmbedCardProps) {
  return null;
});
