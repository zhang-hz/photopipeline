import { create } from "zustand";
import type { AppSettings } from "../types/settings";

const DEFAULT_SETTINGS: AppSettings = {
  theme: "dark",
  language: "English",
  maxRecentFiles: 10,
  checkUpdates: true,
  telemetry: false,
  serverPath: "photopipeline",
  port: 50051,
  autoStart: true,
  gpuBackend: "Auto",
  logLevel: "Info",
  defaultFormat: "HEIF",
  defaultDirectory: "",
  jpegQuality: 95,
  embedMetadata: true,
  thumbnailSize: 120,
  tileSize: 1024,
  cacheDirectory: "",
  maxCacheSize: 1024,
  exifToolPath: "exiftool",
};

interface SettingsState {
  settings: AppSettings;
  snapshot: AppSettings | null;
  isDirty: boolean;
  isLoading: boolean;
}

interface SettingsActions {
  load: () => Promise<void>;
  save: () => Promise<void>;
  reset: () => void;
  cancel: () => void;
  update: <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => void;
}

export const useSettingsStore = create<SettingsState & SettingsActions>((set, get) => ({
  settings: { ...DEFAULT_SETTINGS },
  snapshot: null,
  isDirty: false,
  isLoading: false,

  load: async () => {
    set({ isLoading: true });
    // TODO: invoke("load_settings")
    set({ settings: { ...DEFAULT_SETTINGS }, snapshot: { ...DEFAULT_SETTINGS }, isLoading: false });
  },

  save: async () => {
    // TODO: invoke("save_settings", { settings: get().settings })
    set({ isDirty: false, snapshot: null });
  },

  reset: () =>
    set({ settings: { ...DEFAULT_SETTINGS }, isDirty: true }),

  cancel: () =>
    set((s) => ({
      settings: s.snapshot ? { ...s.snapshot } : { ...DEFAULT_SETTINGS },
      snapshot: null,
      isDirty: false,
    })),

  update: (key, value) =>
    set((s) => ({
      settings: { ...s.settings, [key]: value },
      isDirty: true,
    })),
}));
