export const CHAT_KEYFRAMES = `
@keyframes tg-orb-drift {
  0%   { transform: translate3d(0,0,0) scale(1); }
  33%  { transform: translate3d(3%,4%,0) scale(1.08); }
  66%  { transform: translate3d(-3%,2%,0) scale(1.04); }
  100% { transform: translate3d(0,0,0) scale(1); }
}
@keyframes tg-orb-drift-lite {
  0%   { transform: translate3d(0,0,0); }
  33%  { transform: translate3d(3%,4%,0); }
  66%  { transform: translate3d(-3%,2%,0); }
  100% { transform: translate3d(0,0,0); }
}
@keyframes chat-eq {
  0%, 100% { transform: scaleY(0.35); }
  50%      { transform: scaleY(1); }
}
@keyframes chat-live {
  from { transform: scale(1); opacity: 0.6; }
  to   { transform: scale(1.7); opacity: 0; }
}
@keyframes chat-slide-in {
  from { transform: translate3d(24px,0,0); opacity: 0; }
  to   { transform: translate3d(0,0,0); opacity: 1; }
}
@keyframes chat-pop {
  from { transform: translate3d(0,12px,0) scale(0.97); opacity: 0; }
  to   { transform: translate3d(0,0,0) scale(1); opacity: 1; }
}
@media (prefers-reduced-motion: reduce) {
  .chat-motion { animation: none !important; }
}
`;
