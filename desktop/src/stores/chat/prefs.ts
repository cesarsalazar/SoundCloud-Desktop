import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import type { ConvId } from '../../lib/chat/types';
import { tauriStorage } from '../../lib/tauri-storage';

export const CHAT_HOTKEY_DEFAULT = 'Alt+KeyC';
export const CHAT_DOCK_DEFAULT = 360;
export const CHAT_DOCK_MIN = 320;
export const CHAT_DOCK_MAX = 480;
export const CHAT_DOCK_OVERLAY_BELOW = 1100;
export const CHAT_TABS_MAX = 4;

export interface ChatPrefsState {
  hotkey: string;
  dockOpen: boolean;
  dockWidth: number;
  tabs: ConvId[];
  setHotkey: (hotkey: string) => void;
  setDockOpen: (dockOpen: boolean) => void;
  setDockWidth: (dockWidth: number) => void;
  setTabs: (tabs: ConvId[]) => void;
}

export const useChatPrefsStore = create<ChatPrefsState>()(
  persist(
    (set) => ({
      hotkey: CHAT_HOTKEY_DEFAULT,
      dockOpen: false,
      dockWidth: CHAT_DOCK_DEFAULT,
      tabs: [],
      setHotkey: (hotkey) => set({ hotkey }),
      setDockOpen: (dockOpen) => set({ dockOpen }),
      setDockWidth: (dockWidth) =>
        set({ dockWidth: Math.min(CHAT_DOCK_MAX, Math.max(CHAT_DOCK_MIN, dockWidth)) }),
      setTabs: (tabs) => set({ tabs: tabs.slice(0, CHAT_TABS_MAX) }),
    }),
    {
      name: 'sc-chat-prefs',
      version: 1,
      storage: createJSONStorage(() => tauriStorage),
      partialize: ({ hotkey, dockOpen, dockWidth, tabs }) => ({
        hotkey,
        dockOpen,
        dockWidth,
        tabs,
      }),
    },
  ),
);
