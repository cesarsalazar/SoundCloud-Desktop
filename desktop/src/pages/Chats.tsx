import { memo, type ReactNode } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { ConversationList } from '../components/chat/list/ConversationList';
import { ChatGate } from '../components/chat/shell/ChatGate';
import { ChatsGlow } from '../components/chat/shell/ChatsGlow';
import { EmptyThread } from '../components/chat/shell/EmptyThread';
import { ThreadSlot } from '../components/chat/shell/ThreadSlot';
import { chatPath } from '../components/chat/shell/useOpenChat';
import { usePerfMode } from '../lib/perf';
import { useChatRoomStore } from '../stores/chat/room';

const PANEL =
  'relative flex min-h-0 flex-col overflow-hidden rounded-[1.75rem] border border-white/[0.08]';

function Panel({ className, children }: { className: string; children: ReactNode }) {
  const blur = usePerfMode().blur(40);
  return (
    <section
      className={`${PANEL} ${className}`}
      style={{ boxShadow: 'inset 0 1px 0 rgba(255,255,255,0.07)' }}
    >
      <div
        aria-hidden
        className="absolute inset-0"
        style={{
          contain: 'strict',
          transform: 'translateZ(0)',
          background:
            blur > 0
              ? 'linear-gradient(165deg, rgba(255,255,255,0.06), rgba(255,255,255,0.015) 60%), rgba(14,14,18,0.42)'
              : 'rgba(18,18,22,0.92)',
          backdropFilter: blur > 0 ? `blur(${blur}px) saturate(1.4)` : undefined,
          WebkitBackdropFilter: blur > 0 ? `blur(${blur}px) saturate(1.4)` : undefined,
        }}
      />
      <div className="relative flex min-h-0 flex-1 flex-col" style={{ isolation: 'isolate' }}>
        {children}
      </div>
    </section>
  );
}

export const Chats = memo(function Chats() {
  const { id } = useParams();
  const navigate = useNavigate();
  const selected = id ?? null;
  const roomChat = useChatRoomStore((state) => state.conversationId);
  const open = (conversationId: string) => navigate(chatPath(conversationId));

  return (
    <div className="relative h-full w-full">
      <ChatsGlow />
      <div className="@container relative z-10 mx-auto flex h-full max-w-[1480px] px-4 pt-4 md:px-5 md:pt-5">
        <ChatGate>
          <div className="flex h-full min-h-0 w-full gap-4">
            <Panel
              className={`w-full shrink-0 @min-[641px]:w-[clamp(280px,32cqw,380px)] ${
                selected ? '@max-[640px]:hidden' : ''
              }`}
            >
              <ConversationList
                variant="page"
                selectedId={selected}
                onOpen={open}
                onOpenRoom={() => roomChat && open(roomChat)}
              />
            </Panel>
            <Panel className={`min-w-0 flex-1 ${selected ? '' : '@max-[640px]:hidden'}`}>
              {selected ? (
                <ThreadSlot
                  key={selected}
                  conversationId={selected}
                  variant="page"
                  onBack={() => navigate('/chats')}
                />
              ) : (
                <EmptyThread />
              )}
            </Panel>
          </div>
        </ChatGate>
      </div>
    </div>
  );
});
