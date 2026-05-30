import { create } from "zustand";
import type { DAGNodeData, DAGEdgeData } from "../types/pipeline";

interface PipelineSnapshot {
  nodes: DAGNodeData[];
  edges: DAGEdgeData[];
}

interface PipelineState {
  nodes: DAGNodeData[];
  edges: DAGEdgeData[];
  selectedNodeId: string | null;
  zoom: number;
  panOffset: { x: number; y: number };
  isDirty: boolean;
  executionState: "idle" | "running" | "paused" | "error";
  nodeExecutionStates: Map<string, "pending" | "running" | "done" | "error">;
  undoStack: PipelineSnapshot[];
  redoStack: PipelineSnapshot[];
  currentFilePath: string | null;
  /** Internal: plugin being dragged from PluginBrowser to DAG canvas */
  _draggedPluginId: string | null;
}

interface PipelineActions {
  addNode: (pluginId: string, position?: { x: number; y: number }) => string;
  removeNode: (id: string) => void;
  moveNode: (id: string, position: { x: number; y: number }) => void;
  connectEdge: (fromNodeId: string, toNodeId: string) => boolean;
  removeEdge: (edgeId: string) => void;
  selectNode: (id: string | null) => void;
  newPipeline: () => void;
  markDirty: () => void;
  setZoom: (zoom: number) => void;
  setPan: (offset: { x: number; y: number }) => void;
  undo: () => void;
  redo: () => void;
}

let nodeCounter = 0;

function pushSnapshot(state: PipelineState): PipelineSnapshot {
  return { nodes: structuredClone(state.nodes), edges: structuredClone(state.edges) };
}

export const usePipelineStore = create<PipelineState & PipelineActions>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,
  zoom: 1.0,
  panOffset: { x: 0, y: 0 },
  isDirty: false,
  executionState: "idle",
  nodeExecutionStates: new Map(),
  undoStack: [],
  redoStack: [],
  currentFilePath: null,
  _draggedPluginId: null,

  addNode: (pluginId, position) => {
    const id = `node_${++nodeCounter}`;
    set((s) => {
      const snapshot = pushSnapshot(s);
      const node: DAGNodeData = {
        id,
        pluginId,
        label: pluginId,
        enabled: true,
        position: position ?? { x: 100 + s.nodes.length * 200, y: 60 + s.nodes.length * 30 },
        params: {},
        inputs: [id + "_in"],
        outputs: [id + "_out"],
      };
      const undoStack = [...s.undoStack, snapshot].slice(-50);
    return { nodes: [...s.nodes, node], undoStack, redoStack: [], isDirty: true };
    });
    return id;
  },

  removeNode: (id) =>
    set((s) => {
      const snapshot = pushSnapshot(s);
      return {
        nodes: s.nodes.filter((n) => n.id !== id),
        edges: s.edges.filter((e) => e.fromNode !== id && e.toNode !== id),
        selectedNodeId: s.selectedNodeId === id ? null : s.selectedNodeId,
        undoStack: [...s.undoStack, snapshot].slice(-50),
        redoStack: [],
        isDirty: true,
      };
    }),

  moveNode: (id, position) =>
    set((s) => ({
      nodes: s.nodes.map((n) => (n.id === id ? { ...n, position } : n)),
    })),

  connectEdge: (fromNodeId, toNodeId) => {
    // Client-side cycle detection via BFS
    const { nodes, edges } = get();
    const hasCycle = (from: string, to: string): boolean => {
      const visited = new Set<string>();
      const queue = [to];
      while (queue.length > 0) {
        const current = queue.shift()!;
        if (current === from) return true;
        if (visited.has(current)) continue;
        visited.add(current);
        edges
          .filter((e) => e.fromNode === current)
          .forEach((e) => queue.push(e.toNode));
      }
      return false;
    };

    if (fromNodeId === toNodeId || hasCycle(fromNodeId, toNodeId)) return false;

    set((s) => {
      const snapshot = pushSnapshot(s);
      const edge: DAGEdgeData = { id: `edge_${fromNodeId}_${toNodeId}`, fromNode: fromNodeId, toNode: toNodeId };
      return { edges: [...s.edges, edge], undoStack: [...s.undoStack, snapshot].slice(-50), redoStack: [], isDirty: true };
    });
    return true;
  },

  removeEdge: (edgeId) =>
    set((s) => ({ edges: s.edges.filter((e) => e.id !== edgeId), isDirty: true })),

  selectNode: (id) => set({ selectedNodeId: id }),

  newPipeline: () =>
    set({
      nodes: [],
      edges: [],
      selectedNodeId: null,
      isDirty: false,
      undoStack: [],
      redoStack: [],
      currentFilePath: null,
    }),

  markDirty: () => set({ isDirty: true }),

  setZoom: (zoom) => set({ zoom: Math.max(0.1, Math.min(5.0, zoom)) }),
  setPan: (panOffset) => set({ panOffset }),

  undo: () =>
    set((s) => {
      if (s.undoStack.length === 0) return s;
      const snapshot = s.undoStack[s.undoStack.length - 1];
      const current = pushSnapshot(s);
      return {
        nodes: snapshot.nodes,
        edges: snapshot.edges,
        undoStack: s.undoStack.slice(0, -1),
        redoStack: [...s.redoStack, current],
      };
    }),

  redo: () =>
    set((s) => {
      if (s.redoStack.length === 0) return s;
      const snapshot = s.redoStack[s.redoStack.length - 1];
      const current = pushSnapshot(s);
      return {
        nodes: snapshot.nodes,
        edges: snapshot.edges,
        undoStack: [...s.undoStack, current],
        redoStack: s.redoStack.slice(0, -1),
      };
    }),
}));
