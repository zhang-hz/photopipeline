import { create } from "zustand";

export type AppMode = "edit" | "batch";
export type ThemeMode = "dark" | "light" | "system";

interface AppState {
  mode: AppMode;
  theme: ThemeMode;
  isBackendConnected: boolean;
  statusMessage: string;
}

interface AppActions {
  setMode: (mode: AppMode) => void;
  setTheme: (theme: ThemeMode) => void;
  setBackendStatus: (connected: boolean) => void;
  setStatusMessage: (msg: string) => void;
}

export const useAppStore = create<AppState & AppActions>((set) => ({
  mode: "edit",
  theme: "dark",
  isBackendConnected: false,
  statusMessage: "Ready",

  setMode: (mode) => set({ mode }),
  setTheme: (theme) => set({ theme }),
  setBackendStatus: (connected) =>
    set({ isBackendConnected: connected, statusMessage: connected ? "Ready" : "Backend disconnected" }),
  setStatusMessage: (statusMessage) => set({ statusMessage }),
}));
