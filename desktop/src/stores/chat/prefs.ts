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
export const DOCK_LIST = 'list';
export const DOCK_ROOM = 'room';

export type DockView = typeof DOCK_LIST | typeof DOCK_ROOM | ConvId;

export interface ChatPrefsState {
  hotkey: string;
  dockOpen: boolean;
  dockWidth: number;
  tabs: ConvId[];
  active: DockView;
  setHotkey: (hotkey: string) => void;
  setDockOpen: (dockOpen: boolean) => void;
  setDockWidth: (dockWidth: number) => void;
  setTabs: (tabs: ConvId[]) => void;
  toggleDock: () => void;
  openInDock: (conversationId: ConvId) => void;
  closeTab: (conversationId: ConvId) => void;
  show: (view: DockView) => void;
}

function withTab(tabs: ConvId[], id: ConvId): ConvId[] {
  if (tabs.includes(id)) return tabs;
  return [id, ...tabs].slice(0, CHAT_TABS_MAX);
}

export function clampDockWidth(width: number): number {
  return Math.round(Math.min(CHAT_DOCK_MAX, Math.max(CHAT_DOCK_MIN, width)));
}

export const useChatPrefsStore = create<ChatPrefsState>()(
  persist(
    (set) => ({
      hotkey: CHAT_HOTKEY_DEFAULT,
      dockOpen: false,
      dockWidth: CHAT_DOCK_DEFAULT,
      tabs: [],
      active: DOCK_LIST,
      setHotkey: (hotkey) => set({ hotkey }),
      setDockOpen: (dockOpen) => set({ dockOpen }),
      setDockWidth: (dockWidth) => set({ dockWidth: clampDockWidth(dockWidth) }),
      setTabs: (tabs) => set({ tabs: tabs.slice(0, CHAT_TABS_MAX) }),
      toggleDock: () => set((state) => ({ dockOpen: !state.dockOpen })),
      openInDock: (id) =>
        set((state) => ({ tabs: withTab(state.tabs, id), active: id, dockOpen: true })),
      closeTab: (id) =>
        set((state) => ({
          tabs: state.tabs.filter((tab) => tab !== id),
          active: state.active === id ? DOCK_LIST : state.active,
        })),
      show: (active) => set({ active, dockOpen: true }),
    }),
    {
      name: 'sc-chat-prefs',
      version: 1,
      storage: createJSONStorage(() => tauriStorage),
      partialize: ({ hotkey, dockOpen, dockWidth, tabs, active }) => ({
        hotkey,
        dockOpen,
        dockWidth,
        tabs,
        active,
      }),
    },
  ),
);
