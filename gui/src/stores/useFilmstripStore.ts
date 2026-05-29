import { create } from "zustand";
import type { ImageInfo } from "../types/image";

export type SortKey = "name" | "size" | "format" | "iso";
export type ThumbnailSize = "S" | "M" | "L";

export interface ImageGroup {
  name: string;
  condition?: string;
  color: string;
  imageIndices: Set<number>;
}

interface FilmstripState {
  images: ImageInfo[];
  selectedIndices: Set<number>;
  selectionAnchor: number | null;
  groups: ImageGroup[];
  sortKey: SortKey;
  thumbnailSize: ThumbnailSize;
  thumbnails: Map<number, string>;
  isLoading: boolean;
}

interface FilmstripActions {
  importImages: (images: ImageInfo[]) => void;
  removeImages: (indices: number[]) => void;
  toggleSelect: (index: number, ctrlKey: boolean, shiftKey: boolean) => void;
  selectAll: () => void;
  clearSelection: () => void;
  invertSelection: () => void;
  createGroup: (name: string, condition?: string) => void;
  deleteGroup: (name: string) => void;
  setSortKey: (key: SortKey) => void;
  setThumbnailSize: (size: ThumbnailSize) => void;
  setThumbnail: (index: number, dataUrl: string) => void;
  setLoading: (loading: boolean) => void;
}

const GROUP_COLORS = ["#d59900", "#479ef5", "#54b054", "#ec4899", "#8b5cf6", "#f97316"];

export const useFilmstripStore = create<FilmstripState & FilmstripActions>((set, get) => ({
  images: [],
  selectedIndices: new Set(),
  selectionAnchor: null,
  groups: [],
  sortKey: "name",
  thumbnailSize: "M",
  thumbnails: new Map(),
  isLoading: false,

  importImages: (images) =>
    set((s) => ({ images: [...s.images, ...images], isLoading: false })),

  removeImages: (indices) =>
    set((s) => {
      const idxSet = new Set(indices);
      const newImages = s.images.filter((_, i) => !idxSet.has(i));
      const newSel = new Set<number>();
      s.selectedIndices.forEach((i) => {
        const offset = [...idxSet].filter((x) => x < i).length;
        if (!idxSet.has(i)) newSel.add(i - offset);
      });
      return { images: newImages, selectedIndices: newSel };
    }),

  toggleSelect: (index, ctrlKey, shiftKey) =>
    set((s) => {
      const newSel = new Set(s.selectedIndices);
      if (ctrlKey || (shiftKey && s.selectionAnchor === null)) {
        if (newSel.has(index)) newSel.delete(index);
        else newSel.add(index);
        return { selectedIndices: newSel, selectionAnchor: index };
      }
      if (shiftKey && s.selectionAnchor !== null) {
        const [from, to] = [s.selectionAnchor, index].sort((a, b) => a - b);
        for (let i = from; i <= to; i++) newSel.add(i);
        return { selectedIndices: newSel };
      }
      return { selectedIndices: new Set([index]), selectionAnchor: index };
    }),

  selectAll: () =>
    set((s) => ({
      selectedIndices: new Set(s.images.map((_, i) => i)),
      selectionAnchor: 0,
    })),

  clearSelection: () => set({ selectedIndices: new Set(), selectionAnchor: null }),

  invertSelection: () =>
    set((s) => {
      const newSel = new Set<number>();
      s.images.forEach((_, i) => {
        if (!s.selectedIndices.has(i)) newSel.add(i);
      });
      return { selectedIndices: newSel };
    }),

  createGroup: (name, condition) =>
    set((s) => ({
      groups: [
        ...s.groups,
        {
          name,
          condition,
          color: GROUP_COLORS[s.groups.length % GROUP_COLORS.length],
          imageIndices: new Set(s.selectedIndices),
        },
      ],
    })),

  deleteGroup: (name) =>
    set((s) => ({ groups: s.groups.filter((g) => g.name !== name) })),

  setSortKey: (sortKey) => set({ sortKey }),
  setThumbnailSize: (thumbnailSize) => set({ thumbnailSize }),

  setThumbnail: (index, dataUrl) =>
    set((s) => {
      const newThumbs = new Map(s.thumbnails);
      newThumbs.set(index, dataUrl);
      return { thumbnails: newThumbs };
    }),

  setLoading: (isLoading) => set({ isLoading }),
}));
