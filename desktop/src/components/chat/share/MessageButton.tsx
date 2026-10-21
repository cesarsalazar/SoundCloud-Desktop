import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { startDirectChat } from '../../../lib/chat/share';
import { Loader2, MessageCircle } from '../../../lib/icons';
import { usePerfMode } from '../../../lib/perf';
import { DOWNLOAD_PAGE_URL } from '../../../lib/update-assets';
import { selectChatEnabled, useChatStore } from '../../../stores/chat';

interface MessageButtonProps {
  userUrn: string;
  name: string;
}

function errorCode(error: unknown): string | null {
  if (typeof error !== 'object' || error === null) return null;
  const code = (error as { code?: unknown }).code;
  return typeof code === 'string' ? code : null;
}

async function copyInvite(text: string, done: string, failed: string) {
  try {
    await navigator.clipboard.writeText(text);
    toast.success(done);
  } catch {
    toast.error(failed);
  }
}

export function MessageButton({ userUrn, name }: MessageButtonProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const blur = usePerfMode().blur(20);
  const enabled = useChatStore(selectChatEnabled);
  const [opening, setOpening] = useState(false);
  if (!enabled) return null;

  const invite = () =>
    void copyInvite(
      t('chat.embeds.message.inviteText', { url: DOWNLOAD_PAGE_URL }),
      t('chat.embeds.message.inviteCopied'),
      t('chat.embeds.message.copyFailed'),
    );

  const open = async () => {
    setOpening(true);
    try {
      const id = await startDirectChat(userUrn);
      if (id) navigate(`/chats/${encodeURIComponent(id)}`);
    } catch (error) {
      const unavailable = errorCode(error) === 'unavailable';
      toast.error(
        unavailable
          ? t('chat.embeds.message.unavailable', { name })
          : t('chat.embeds.message.failed'),
        unavailable
          ? { action: { label: t('chat.embeds.message.invite'), onClick: invite } }
          : undefined,
      );
    } finally {
      setOpening(false);
    }
  };

  return (
    <button
      type="button"
      disabled={opening}
      onClick={() => void open()}
      className={`inline-flex h-11 items-center gap-2 rounded-full border-[0.5px] border-white/[0.1] px-5 text-[13px] font-semibold text-white/85 transition-all duration-300 ease-[var(--ease-apple)] cursor-pointer hover:border-white/[0.18] hover:text-white active:scale-[0.97] disabled:cursor-default ${
        blur > 0
          ? 'bg-white/[0.06] hover:bg-white/[0.1]'
          : 'bg-[rgba(36,36,42,0.9)] hover:bg-[rgba(48,48,56,0.92)]'
      }`}
      style={{
        backdropFilter: blur > 0 ? `blur(${blur}px)` : undefined,
        WebkitBackdropFilter: blur > 0 ? `blur(${blur}px)` : undefined,
      }}
    >
      {opening ? <Loader2 size={15} className="animate-spin" /> : <MessageCircle size={15} />}
      {t('chat.embeds.message.write')}
    </button>
  );
}
