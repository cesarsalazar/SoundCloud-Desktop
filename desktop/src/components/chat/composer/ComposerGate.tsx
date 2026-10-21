import { memo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { chatBridge } from '../../../lib/chat/bridge';
import type { ConvId } from '../../../lib/chat/types';
import { Ban, Check, X } from '../../../lib/icons';
import { useChatThreadStore } from '../../../stores/chat/thread';
import { useThread } from '../thread/context';
import { toastError } from '../thread/errors';
import { isActiveMember, isRequest } from '../thread/permissions';
import { Composer } from './Composer';

function RequestBar({ conversationId, onLeave }: { conversationId: ConvId; onLeave?: () => void }) {
  const { t, i18n } = useTranslation();
  const [busy, setBusy] = useState(false);
  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    try {
      await action();
    } catch (error) {
      toastError(error, t, i18n.language);
    } finally {
      setBusy(false);
    }
  };
  const accept = () =>
    run(async () => {
      const summary = await chatBridge.accept(conversationId);
      useChatThreadStore.getState().setConversation(summary);
    });
  const decline = () =>
    run(async () => {
      await chatBridge.decline(conversationId);
      onLeave?.();
    });

  return (
    <div className="flex shrink-0 justify-center gap-2 px-3 pt-1.5 pb-3">
      <button
        type="button"
        disabled={busy}
        onClick={() => void decline()}
        className="flex h-10 cursor-pointer items-center gap-2 rounded-full border border-white/[0.08] px-5 text-[13px] font-semibold text-white/70 transition-colors duration-150 hover:bg-white/[0.06] disabled:opacity-50"
      >
        <X size={15} />
        {t('chat.thread.request.decline')}
      </button>
      <button
        type="button"
        disabled={busy}
        onClick={() => void accept()}
        className="flex h-10 cursor-pointer items-center gap-2 rounded-full px-6 text-[13px] font-bold transition-transform duration-150 hover:scale-[1.03] disabled:opacity-50"
        style={{ background: 'var(--color-accent)', color: 'var(--color-accent-contrast)' }}
      >
        <Check size={15} />
        {t('chat.thread.request.accept')}
      </button>
    </div>
  );
}

function GoneBar() {
  const { t } = useTranslation();
  return (
    <div className="flex shrink-0 items-center justify-center gap-2 px-3 pt-2 pb-4 text-[12.5px] text-white/40">
      <Ban size={14} />
      {t('chat.thread.notMember')}
    </div>
  );
}

export const ComposerGate = memo(function ComposerGate({
  conversationId,
  onLeave,
}: {
  conversationId: ConvId;
  onLeave?: () => void;
}) {
  const { conversation } = useThread();
  if (isRequest(conversation)) {
    return <RequestBar conversationId={conversationId} onLeave={onLeave} />;
  }
  if (!isActiveMember(conversation)) return <GoneBar />;
  return <Composer conversationId={conversationId} />;
});
