import { create } from "zustand";
import type { ImageInfo } from "../types/image";

export interface BatchItem {
  image: ImageInfo;
  status: "queued" | "processing" | "done" | "failed";
  errorMessage?: string;
}

export interface BatchOutputSettings {
  directory: string;
  template: string;
  format: string;
  quality: number;
  parallel: number;
  conflict: "skip" | "overwrite" | "rename";
}

export interface BatchProgress {
  total: number;
  done: number;
  failed: number;
  elapsedSecs: number;
  etaSecs: number;
  speedPerMin: number;
}

interface BatchState {
  queue: BatchItem[];
  batchState: "idle" | "running" | "paused" | "done";
  batchId: string | null;
  progress: BatchProgress | null;
  outputSettings: BatchOutputSettings;
  perImageOverrides: Map<string, Record<string, unknown>>;
  _startTime: number;
  _elapsedBeforePause: number;
  _timerId: ReturnType<typeof setInterval> | null;
  _processTimeout: ReturnType<typeof setTimeout> | null;
}

interface BatchActions {
  addToQueue: (images: ImageInfo[]) => void;
  removeFromQueue: (indices: number[]) => void;
  clearDone: () => void;
  setOutputSetting: <K extends keyof BatchOutputSettings>(key: K, value: BatchOutputSettings[K]) => void;
  setPerImageOverride: (imageId: string, nodeId: string, paramId: string, value: unknown) => void;
  setBatchState: (state: "idle" | "running" | "paused" | "done") => void;
  startBatch: () => void;
  pauseBatch: () => void;
  resumeBatch: () => void;
  stopBatch: () => void;
  updateProgress: (p: Partial<BatchProgress>) => void;
}

export const useBatchStore = create<BatchState & BatchActions>((set, get) => ({
  queue: [],
  batchState: "idle",
  batchId: null,
  progress: null,
  outputSettings: { directory: "C:\\output", template: "{date}/{filename}", format: "HEIF", quality: 95, parallel: 4, conflict: "skip" },
  perImageOverrides: new Map(),
  _startTime: 0,
  _elapsedBeforePause: 0,
  _timerId: null,
  _processTimeout: null,

  addToQueue: (images) => set((s) => {
    const existingIds = new Set(s.queue.map((item) => item.image.id));
    const newItems: BatchItem[] = images.filter((img) => !existingIds.has(img.id)).map((image) => ({ image, status: "queued" as const }));
    return { queue: [...s.queue, ...newItems] };
  }),

  removeFromQueue: (indices) => set((s) => {
    const idxSet = new Set(indices);
    return { queue: s.queue.filter((_, i) => !idxSet.has(i)) };
  }),

  clearDone: () => set((s) => ({
    queue: s.queue.filter((item) => item.status !== "done" && item.status !== "failed"),
    batchState: s.queue.every((q) => q.status === "done" || q.status === "failed") ? "idle" : s.batchState,
  })),

  setOutputSetting: (key, value) => set((s) => ({ outputSettings: { ...s.outputSettings, [key]: value } })),

  setPerImageOverride: (imageId, nodeId, paramId, value) => set((s) => {
    const newMap = new Map(s.perImageOverrides);
    const imageOverrides = { ...(newMap.get(imageId) ?? {}), [`${nodeId}.${paramId}`]: value };
    newMap.set(imageId, imageOverrides);
    return { perImageOverrides: newMap };
  }),

  setBatchState: (batchState) => set({ batchState }),

  updateProgress: (p) => set((s) => ({ progress: s.progress ? { ...s.progress, ...p } : { total: s.queue.length, done: 0, failed: 0, elapsedSecs: 0, etaSecs: 0, speedPerMin: 0, ...p } })),

  startBatch: () => {
    const s = get();
    if (s.queue.length === 0) return;

    // Clear any pending timers
    if (s._timerId) clearInterval(s._timerId);
    if (s._processTimeout) clearTimeout(s._processTimeout);

    const startTime = Date.now();
    const initialProgress: BatchProgress = { total: s.queue.length, done: 0, failed: 0, elapsedSecs: 0, etaSecs: 0, speedPerMin: 0 };

    set({ batchState: "running", _startTime: startTime, _elapsedBeforePause: 0, progress: initialProgress });

    // Timer to update elapsed time and metrics
    const timerId = setInterval(() => {
      const st = get();
      if (st.batchState !== "running") return;
      const elapsed = (Date.now() - st._startTime) / 1000 + st._elapsedBeforePause;
      const done = st.queue.filter((q) => q.status === "done" || q.status === "failed").length;
      const failed = st.queue.filter((q) => q.status === "failed").length;
      const pct = st.queue.length > 0 ? done / st.queue.length : 0;
      const speed = elapsed > 0 ? (done / (elapsed / 60)) : 0;
      const eta = pct > 0.01 ? (elapsed / pct) * (1 - pct) : 0;
      set({ progress: { total: st.queue.length, done: done - failed, failed, elapsedSecs: Math.round(elapsed), etaSecs: Math.round(eta), speedPerMin: Math.round(speed * 10) / 10 } });
    }, 500);
    set({ _timerId: timerId });

    // Process queue items sequentially
    const processNext = (index: number) => {
      const st = get();
      if (st.batchState !== "running") { clearInterval(timerId); return; }
      if (index >= st.queue.length) {
        clearInterval(timerId);
        set({ batchState: "done", _timerId: null });
        return;
      }

      const item = st.queue[index];
      if (item.status === "queued") {
        set((prev) => ({ queue: prev.queue.map((q, i) => i === index ? { ...q, status: "processing" as const } : q) }));
        const delay = 300 + Math.random() * 700;
        const timeout = setTimeout(() => {
          const s2 = get();
          if (s2.batchState !== "running") return;
          const isError = Math.random() < 0.08;
          set((prev) => ({ queue: prev.queue.map((q, i) => i === index ? { ...q, status: (isError ? "failed" : "done") as const, errorMessage: isError ? "Encode error: output path not writable" : undefined } : q) }));
          processNext(index + 1);
        }, delay);
        set({ _processTimeout: timeout });
      } else {
        processNext(index + 1);
      }
    };
    processNext(0);
  },

  pauseBatch: () => {
    const s = get();
    if (s._timerId) clearInterval(s._timerId);
    if (s._processTimeout) clearTimeout(s._processTimeout);
    const elapsed = (Date.now() - s._startTime) / 1000 + s._elapsedBeforePause;
    set({ batchState: "paused", _elapsedBeforePause: elapsed, _timerId: null, _processTimeout: null });
  },

  resumeBatch: () => {
    const s = get();
    set({ batchState: "running", _startTime: Date.now() });
    // Re-start timer
    const timerId = setInterval(() => {
      const st = get();
      if (st.batchState !== "running") { clearInterval(timerId); return; }
      const elapsed = (Date.now() - st._startTime) / 1000 + st._elapsedBeforePause;
      const done = st.queue.filter((q) => q.status === "done" || q.status === "failed").length;
      const failed = st.queue.filter((q) => q.status === "failed").length;
      const pct = st.queue.length > 0 ? done / st.queue.length : 0;
      const speed = elapsed > 0 ? (done / (elapsed / 60)) : 0;
      const eta = pct > 0.01 ? (elapsed / pct) * (1 - pct) : 0;
      set({ progress: { total: st.queue.length, done: done - failed, failed, elapsedSecs: Math.round(elapsed), etaSecs: Math.round(eta), speedPerMin: Math.round(speed * 10) / 10 } });
    }, 500);
    set({ _timerId: timerId });
    // Re-start processing from where we left off
    const nextQueued = s.queue.findIndex((q) => q.status === "queued" || q.status === "processing");
    if (nextQueued >= 0) {
      const processNext = (index: number) => {
        const st = get();
        if (st.batchState !== "running") { clearInterval(timerId); return; }
        if (index >= st.queue.length) { clearInterval(timerId); set({ batchState: "done", _timerId: null }); return; }
        const item = st.queue[index];
        if (item.status === "queued") {
          set((prev) => ({ queue: prev.queue.map((q, i) => i === index ? { ...q, status: "processing" as const } : q) }));
          const timeout = setTimeout(() => {
            const s2 = get();
            if (s2.batchState !== "running") return;
            const isError = Math.random() < 0.08;
            set((prev) => ({ queue: prev.queue.map((q, i) => i === index ? { ...q, status: (isError ? "failed" : "done") as const, errorMessage: isError ? "Encode error" : undefined } : q) }));
            processNext(index + 1);
          }, 300 + Math.random() * 700);
          set({ _processTimeout: timeout });
        } else {
          processNext(index + 1);
        }
      };
      processNext(nextQueued);
    }
  },

  stopBatch: () => {
    const s = get();
    if (s._timerId) clearInterval(s._timerId);
    if (s._processTimeout) clearTimeout(s._processTimeout);
    set((prev) => ({
      batchState: "idle",
      _timerId: null,
      _processTimeout: null,
      queue: prev.queue.map((q) => q.status === "queued" || q.status === "processing" ? { ...q, status: "failed" as const, errorMessage: "Cancelled by user" } : q),
    }));
  },
}));
