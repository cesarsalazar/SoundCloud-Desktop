import { create } from 'zustand';
import type { Device, DeviceId } from '../../lib/chat/types';

export interface ChatDevicesState {
  alerts: Device[];
  pushAlert: (device: Device) => void;
  dismissAlert: (deviceId: DeviceId) => void;
}

export const useChatDevicesStore = create<ChatDevicesState>()((set) => ({
  alerts: [],
  pushAlert: (device) =>
    set((state) => ({
      alerts: [...state.alerts.filter((alert) => alert.id !== device.id), device],
    })),
  dismissAlert: (deviceId) =>
    set((state) => ({ alerts: state.alerts.filter((alert) => alert.id !== deviceId) })),
}));
