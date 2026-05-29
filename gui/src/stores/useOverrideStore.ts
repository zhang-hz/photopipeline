import { create } from "zustand";

export type OverrideScope = "all" | "template" | "group" | "image";
export type OverrideSource = "plugin_default" | "template" | "group" | "image" | "expression";

export interface OverrideEntry {
  value: unknown;
  source: OverrideSource;
  sourceName?: string;
}

export interface ValueWithSource {
  value: unknown;
  source: OverrideSource;
  sourceName?: string;
  isOverridden: boolean;
  isExpression: boolean;
  isEditable: boolean;
}

interface OverrideState {
  scope: OverrideScope;
  activeGroupName: string | null;
  activeImageIndex: number | null;
  overrides: Map<string, OverrideEntry>;
  expressions: Map<string, string>;
  dirtyParams: Set<string>;
}

interface OverrideActions {
  setScope: (scope: OverrideScope, groupName?: string, imageIndex?: number) => void;
  initForNode: (nodeId: string) => void;
  setOverride: (nodeId: string, paramId: string, value: unknown) => void;
  clearOverride: (nodeId: string, paramId: string) => void;
  setExpression: (nodeId: string, paramId: string, expr: string) => void;
  getEffectiveValue: (nodeId: string, paramId: string) => ValueWithSource;
  getSectionOverrideInfo: (nodeId: string, sectionId: string) => { badge: string; hasOverrides: boolean };
  hasVaryingValues: (nodeId: string, paramId: string) => boolean;
}

export const useOverrideStore = create<OverrideState & OverrideActions>((set, get) => ({
  scope: "template",
  activeGroupName: null,
  activeImageIndex: null,
  overrides: new Map(),
  expressions: new Map(),
  dirtyParams: new Set(),

  setScope: (scope, groupName, imageIndex) =>
    set({ scope, activeGroupName: groupName ?? null, activeImageIndex: imageIndex ?? null }),

  initForNode: (_nodeId: string) => {
    // Initialize override state for a newly selected node
    // TODO: load overrides from pipeline config
  },

  setOverride: (nodeId, paramId, value) =>
    set((s) => {
      const key = `${nodeId}.${paramId}`;
      const newOverrides = new Map(s.overrides);
      newOverrides.set(key, { value, source: s.scope as OverrideSource, sourceName: s.activeGroupName ?? undefined });
      const newDirty = new Set(s.dirtyParams);
      newDirty.add(key);
      return { overrides: newOverrides, dirtyParams: newDirty };
    }),

  clearOverride: (nodeId, paramId) =>
    set((s) => {
      const key = `${nodeId}.${paramId}`;
      const newOverrides = new Map(s.overrides);
      newOverrides.delete(key);
      return { overrides: newOverrides };
    }),

  setExpression: (nodeId, paramId, expr) =>
    set((s) => {
      const key = `${nodeId}.${paramId}`;
      const newExpr = new Map(s.expressions);
      newExpr.set(key, expr);
      return { expressions: newExpr };
    }),

  getEffectiveValue: (nodeId: string, paramId: string): ValueWithSource => {
    const { overrides, expressions, scope } = get();
    const key = `${nodeId}.${paramId}`;

    // Check expression first
    if (expressions.has(key)) {
      return { value: `expr:${expressions.get(key)}`, source: "expression", isOverridden: true, isExpression: true, isEditable: scope !== "all" };
    }

    // Check override
    const override = overrides.get(key);
    if (override) {
      return { ...override, isOverridden: true, isExpression: false, isEditable: scope !== "all" };
    }

    // TODO: resolve from template/group/image/plugin_default hierarchy
    return { value: null, source: "plugin_default", isOverridden: false, isExpression: false, isEditable: scope !== "all" };
  },

  getSectionOverrideInfo: (_nodeId: string, _sectionId: string) => {
    // TODO: count overrides in section
    return { badge: "inherited", hasOverrides: false };
  },

  hasVaryingValues: (_nodeId: string, _paramId: string) => {
    // TODO: check if values differ across selected images
    return false;
  },
}));
