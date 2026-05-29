import { describe, it, expect, beforeEach, vi } from "vitest";
import { useBatchStore, type BatchItem } from "../useBatchStore";
import type { ImageInfo } from "../../types/image";

const makeImg = (id: string, filename: string): ImageInfo => ({
  id, path: `/p/${filename}`, filename, format: "raw",
  width: 6000, height: 4000, file_size_bytes: 50000000,
  pixel_format: "u16", color_space: "srgb",
});

describe("useBatchStore", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    useBatchStore.setState({
      queue: [], batchState: "idle", batchId: null, progress: null,
      outputSettings: { directory: "", template: "{date}/{filename}", format: "HEIF", quality: 95, parallel: 4, conflict: "skip" },
      perImageOverrides: new Map(),
      _startTime: 0, _elapsedBeforePause: 0, _timerId: null, _processTimeout: null,
    });
  });

  afterEach(() => { vi.useRealTimers(); });

  describe("addToQueue", () => {
    it("should add images to queue with queued status", () => {
      const imgs = [makeImg("1", "a.ARW"), makeImg("2", "b.ARW")];
      useBatchStore.getState().addToQueue(imgs);
      expect(useBatchStore.getState().queue).toHaveLength(2);
      expect(useBatchStore.getState().queue[0].status).toBe("queued");
    });

    it("should deduplicate by image id", () => {
      const imgs = [makeImg("1", "a.ARW")];
      useBatchStore.getState().addToQueue(imgs);
      useBatchStore.getState().addToQueue(imgs);
      expect(useBatchStore.getState().queue).toHaveLength(1);
    });
  });

  describe("removeFromQueue", () => {
    it("should remove items by index", () => {
      useBatchStore.getState().addToQueue([makeImg("1", "a"), makeImg("2", "b"), makeImg("3", "c")]);
      useBatchStore.getState().removeFromQueue([1]);
      expect(useBatchStore.getState().queue).toHaveLength(2);
      expect(useBatchStore.getState().queue[0].image.id).toBe("1");
      expect(useBatchStore.getState().queue[1].image.id).toBe("3");
    });
  });

  describe("clearDone", () => {
    it("should remove done and failed items", () => {
      useBatchStore.getState().addToQueue([makeImg("1", "a"), makeImg("2", "b")]);
      useBatchStore.setState((s) => ({
        queue: s.queue.map((q, i) => ({ ...q, status: i === 0 ? "done" as const : "queued" as const })),
      }));
      useBatchStore.getState().clearDone();
      expect(useBatchStore.getState().queue).toHaveLength(1);
      expect(useBatchStore.getState().queue[0].status).toBe("queued");
    });
  });

  describe("outputSettings", () => {
    it("should update individual settings", () => {
      useBatchStore.getState().setOutputSetting("format", "JXL");
      expect(useBatchStore.getState().outputSettings.format).toBe("JXL");
    });
  });

  describe("perImageOverride", () => {
    it("should set and get per-image overrides", () => {
      useBatchStore.getState().setPerImageOverride("img1", "node1", "strength", 0.8);
      expect(useBatchStore.getState().perImageOverrides.get("img1")).toEqual({ "node1.strength": 0.8 });
    });
  });

  describe("startBatch simulation", () => {
    it("should not start with empty queue", () => {
      useBatchStore.getState().startBatch();
      expect(useBatchStore.getState().batchState).toBe("idle");
    });

    it("should transition to running and process items", () => {
      useBatchStore.getState().addToQueue([makeImg("1", "a.ARW"), makeImg("2", "b.ARW")]);
      useBatchStore.getState().startBatch();
      expect(useBatchStore.getState().batchState).toBe("running");

      // First item should be processing
      expect(useBatchStore.getState().queue[0].status).toBe("processing");

      // Advance timers past processing delay
      vi.advanceTimersByTime(1500);
      expect(useBatchStore.getState().queue[0].status).toMatch(/done|failed/);
    });
  });

  describe("pause/resume", () => {
    it("should pause batch", () => {
      useBatchStore.getState().addToQueue([makeImg("1", "a.ARW")]);
      useBatchStore.getState().startBatch();
      expect(useBatchStore.getState().batchState).toBe("running");
      useBatchStore.getState().pauseBatch();
      expect(useBatchStore.getState().batchState).toBe("paused");
    });

    it("should resume paused batch", () => {
      useBatchStore.getState().addToQueue([makeImg("1", "a.ARW")]);
      useBatchStore.getState().startBatch();
      useBatchStore.getState().pauseBatch();
      useBatchStore.getState().resumeBatch();
      expect(useBatchStore.getState().batchState).toMatch(/running|done/);
    });
  });

  describe("stopBatch", () => {
    it("should cancel queued items", () => {
      useBatchStore.getState().addToQueue([makeImg("1", "a"), makeImg("2", "b")]);
      useBatchStore.getState().startBatch();
      // First item already processing
      useBatchStore.getState().stopBatch();
      expect(useBatchStore.getState().queue[0].status).toBe("failed"); // processing → failed
      expect(useBatchStore.getState().queue[0].errorMessage).toBe("Cancelled by user");
    });
  });
});
