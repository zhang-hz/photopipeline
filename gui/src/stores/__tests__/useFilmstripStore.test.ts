import { describe, it, expect, beforeEach } from "vitest";
import { useFilmstripStore } from "../useFilmstripStore";
import type { ImageInfo } from "../../types/image";

const makeImg = (id: string, filename: string, iso?: number, format = "raw"): ImageInfo => ({
  id, path: `/p/${filename}`, filename, format,
  width: 6000, height: 4000, file_size_bytes: 50000000,
  pixel_format: "u16", color_space: "srgb",
  metadata: iso != null ? { iso, make: "Sony" } as any : undefined,
});

describe("useFilmstripStore", () => {
  beforeEach(() => {
    useFilmstripStore.setState({
      images: [], selectedIndices: new Set(), selectionAnchor: null,
      groups: [], sortKey: "name", thumbnailSize: "M", thumbnails: new Map(), isLoading: false,
    });
  });

  describe("importImages", () => {
    it("should add images", () => {
      useFilmstripStore.getState().importImages([makeImg("1", "a.ARW")]);
      expect(useFilmstripStore.getState().images).toHaveLength(1);
    });

    it("should append to existing images", () => {
      useFilmstripStore.getState().importImages([makeImg("1", "a.ARW")]);
      useFilmstripStore.getState().importImages([makeImg("2", "b.ARW")]);
      expect(useFilmstripStore.getState().images).toHaveLength(2);
    });
  });

  describe("toggleSelect", () => {
    it("should single-select on plain click", () => {
      useFilmstripStore.getState().importImages([makeImg("1", "a"), makeImg("2", "b")]);
      useFilmstripStore.getState().toggleSelect(0, false, false);
      expect(useFilmstripStore.getState().selectedIndices).toEqual(new Set([0]));
    });

    it("should clear previous selection on plain click", () => {
      useFilmstripStore.getState().importImages([makeImg("1", "a"), makeImg("2", "b")]);
      useFilmstripStore.getState().toggleSelect(0, false, false);
      useFilmstripStore.getState().toggleSelect(1, false, false);
      expect(useFilmstripStore.getState().selectedIndices).toEqual(new Set([1]));
    });

    it("should toggle with Ctrl+click", () => {
      useFilmstripStore.getState().importImages([makeImg("1", "a"), makeImg("2", "b")]);
      useFilmstripStore.getState().toggleSelect(0, true, false);
      useFilmstripStore.getState().toggleSelect(1, true, false);
      expect(useFilmstripStore.getState().selectedIndices.size).toBe(2);
      useFilmstripStore.getState().toggleSelect(0, true, false);
      expect(useFilmstripStore.getState().selectedIndices).toEqual(new Set([1]));
    });

    it("should range-select with Shift+click", () => {
      useFilmstripStore.getState().importImages([makeImg("1", "a"), makeImg("2", "b"), makeImg("3", "c")]);
      useFilmstripStore.getState().toggleSelect(0, false, false); // anchor at 0
      useFilmstripStore.getState().toggleSelect(2, false, true); // shift-click to 2
      expect(useFilmstripStore.getState().selectedIndices).toEqual(new Set([0, 1, 2]));
    });
  });

  describe("selectAll / clearSelection / invertSelection", () => {
    beforeEach(() => {
      useFilmstripStore.getState().importImages([makeImg("1", "a"), makeImg("2", "b"), makeImg("3", "c")]);
    });

    it("selectAll should select all", () => {
      useFilmstripStore.getState().selectAll();
      expect(useFilmstripStore.getState().selectedIndices.size).toBe(3);
    });

    it("clearSelection should clear all", () => {
      useFilmstripStore.getState().selectAll();
      useFilmstripStore.getState().clearSelection();
      expect(useFilmstripStore.getState().selectedIndices.size).toBe(0);
    });

    it("invertSelection should invert", () => {
      useFilmstripStore.getState().toggleSelect(0, false, false);
      useFilmstripStore.getState().invertSelection();
      expect(useFilmstripStore.getState().selectedIndices).toEqual(new Set([1, 2]));
    });
  });

  describe("removeImages", () => {
    it("should remove and reindex selection", () => {
      useFilmstripStore.getState().importImages([makeImg("1", "a"), makeImg("2", "b"), makeImg("3", "c")]);
      useFilmstripStore.getState().toggleSelect(1, false, false); // select index 1
      useFilmstripStore.getState().removeImages([0]); // remove index 0
      // Selection should shift: index 1 → index 0
      expect(useFilmstripStore.getState().images).toHaveLength(2);
      expect(useFilmstripStore.getState().selectedIndices).toEqual(new Set([0]));
    });
  });

  describe("groups", () => {
    beforeEach(() => {
      useFilmstripStore.getState().importImages([makeImg("1", "a"), makeImg("2", "b")]);
      useFilmstripStore.getState().selectAll();
    });

    it("should create group with current selection", () => {
      useFilmstripStore.getState().createGroup("Test");
      expect(useFilmstripStore.getState().groups).toHaveLength(1);
      expect(useFilmstripStore.getState().groups[0].name).toBe("Test");
      expect(useFilmstripStore.getState().groups[0].imageIndices.size).toBe(2);
    });

    it("should delete group by name", () => {
      useFilmstripStore.getState().createGroup("Test");
      useFilmstripStore.getState().deleteGroup("Test");
      expect(useFilmstripStore.getState().groups).toHaveLength(0);
    });

    it("should assign unique colors", () => {
      useFilmstripStore.getState().createGroup("A");
      useFilmstripStore.getState().createGroup("B");
      const colors = useFilmstripStore.getState().groups.map((g) => g.color);
      expect(colors[0]).not.toBe(colors[1]);
    });
  });
});
