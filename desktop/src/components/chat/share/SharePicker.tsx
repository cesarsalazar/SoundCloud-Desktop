import { memo } from 'react';
import type { ConvId, Embed } from '../../../lib/chat/types';

export interface SharePickerProps {
  open: boolean;
  embed: Embed | null;
  onClose: () => void;
  onSent?: (conversationIds: ConvId[]) => void;
}

export const SharePicker = memo(function SharePicker(_props: SharePickerProps) {
  return null;
});
