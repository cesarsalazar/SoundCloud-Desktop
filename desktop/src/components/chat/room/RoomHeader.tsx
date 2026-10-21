import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { Hourglass } from '../../../lib/chat/icons';
import type { RoomView } from '../../../lib/together/types';
import { SyncBadge } from '../../together/SyncBadge';
import { RoomAccessToggle } from './RoomAccessToggle';
import { RoomCodeChip } from './RoomCodeChip';
import { RoomLeaveButton } from './RoomLeaveButton';
import { RoomSleeve } from './RoomSleeve';
import { RoomTitle } from './RoomTitle';
import { RoomTuning } from './RoomTuning';
import type { RoomSelf } from './useRoom';

function Ephemeral() {
  const { t } = useTranslation();
  return (
    <span
      title={t('chat.room.ephemeral')}
      aria-label={t('chat.room.ephemeral')}
      className="flex size-7 shrink-0 items-center justify-center rounded-full border border-white/[0.09] bg-black/25 text-amber-200/80"
    >
      <Hourglass size={13} />
    </span>
  );
}

export const RoomHeader = memo(function RoomHeader({
  room,
  self,
  moderation,
}: {
  room: RoomView;
  self: RoomSelf;
  moderation: boolean;
}) {
  return (
    <header className="relative shrink-0 border-b border-white/[0.05]">
      <RoomSleeve room={room} />
      <div className="relative px-3 pt-3 pb-2.5" style={{ isolation: 'isolate' }}>
        <div className="flex items-center gap-2">
          <div className="flex min-w-0 flex-1">
            <RoomTitle room={room} editable={self.host && moderation} />
          </div>
          <Ephemeral />
          {room.public !== undefined && <RoomAccessToggle open={room.public} host={self.host} />}
        </div>
        <div className="mt-2 flex items-center gap-2">
          <RoomCodeChip code={room.code} />
          <div className="flex min-w-0 flex-1">
            <SyncBadge className="min-w-0 max-w-full" />
          </div>
          {!self.host && <RoomTuning />}
          <RoomLeaveButton room={room} host={self.host} selfId={self.id} />
        </div>
      </div>
    </header>
  );
});
