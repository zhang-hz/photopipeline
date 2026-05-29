import { describe, it, expect, beforeEach } from "vitest";
import { usePipelineStore } from "../usePipelineStore";

describe("usePipelineStore", () => {
  beforeEach(() => {
    usePipelineStore.setState({
      nodes: [], edges: [], selectedNodeId: null, zoom: 1, panOffset: { x: 0, y: 0 },
      isDirty: false, executionState: "idle", nodeExecutionStates: new Map(),
      undoStack: [], redoStack: [], currentFilePath: null,
    });
  });

  describe("addNode", () => {
    it("should add a node with unique id", () => {
      const id = usePipelineStore.getState().addNode("photopipeline.plugins.raw_input");
      expect(id).toMatch(/^node_\d+$/);
      expect(usePipelineStore.getState().nodes).toHaveLength(1);
    });

    it("should set default position", () => {
      const id = usePipelineStore.getState().addNode("photopipeline.plugins.transform", { x: 200, y: 100 });
      const node = usePipelineStore.getState().nodes.find((n) => n.id === id);
      expect(node?.position).toEqual({ x: 200, y: 100 });
    });

    it("should mark pipeline as dirty", () => {
      usePipelineStore.getState().addNode("p");
      expect(usePipelineStore.getState().isDirty).toBe(true);
    });

    it("should push undo snapshot", () => {
      usePipelineStore.getState().addNode("a");
      expect(usePipelineStore.getState().undoStack.length).toBe(1);
    });

    it("should clear redo stack on new action", () => {
      usePipelineStore.setState({ redoStack: [{ nodes: [], edges: [] }] });
      usePipelineStore.getState().addNode("a");
      expect(usePipelineStore.getState().redoStack.length).toBe(0);
    });
  });

  describe("removeNode", () => {
    it("should remove node and associated edges", () => {
      const a = usePipelineStore.getState().addNode("a", { x: 0, y: 0 });
      const b = usePipelineStore.getState().addNode("b", { x: 200, y: 0 });
      usePipelineStore.getState().connectEdge(a, b);
      expect(usePipelineStore.getState().edges).toHaveLength(1);

      usePipelineStore.getState().removeNode(a);
      expect(usePipelineStore.getState().nodes).toHaveLength(1);
      expect(usePipelineStore.getState().edges).toHaveLength(0);
      expect(usePipelineStore.getState().nodes[0].id).toBe(b);
    });

    it("should clear selection if selected node is removed", () => {
      const id = usePipelineStore.getState().addNode("a");
      usePipelineStore.getState().selectNode(id);
      usePipelineStore.getState().removeNode(id);
      expect(usePipelineStore.getState().selectedNodeId).toBeNull();
    });
  });

  describe("connectEdge", () => {
    it("should create edge between nodes", () => {
      const a = usePipelineStore.getState().addNode("a", { x: 0, y: 0 });
      const b = usePipelineStore.getState().addNode("b", { x: 200, y: 0 });
      const result = usePipelineStore.getState().connectEdge(a, b);
      expect(result).toBe(true);
      expect(usePipelineStore.getState().edges).toHaveLength(1);
      expect(usePipelineStore.getState().edges[0].fromNode).toBe(a);
      expect(usePipelineStore.getState().edges[0].toNode).toBe(b);
    });

    it("should reject self-loops", () => {
      const a = usePipelineStore.getState().addNode("a");
      const result = usePipelineStore.getState().connectEdge(a, a);
      expect(result).toBe(false);
      expect(usePipelineStore.getState().edges).toHaveLength(0);
    });

    it("should reject cycles (A→B→A)", () => {
      const a = usePipelineStore.getState().addNode("a", { x: 0, y: 0 });
      const b = usePipelineStore.getState().addNode("b", { x: 200, y: 0 });
      usePipelineStore.getState().connectEdge(a, b);
      const result = usePipelineStore.getState().connectEdge(b, a);
      expect(result).toBe(false);
    });

    it("should reject complex cycles (A→B→C→A)", () => {
      const a = usePipelineStore.getState().addNode("a");
      const b = usePipelineStore.getState().addNode("b");
      const c = usePipelineStore.getState().addNode("c");
      usePipelineStore.getState().connectEdge(a, b);
      usePipelineStore.getState().connectEdge(b, c);
      const result = usePipelineStore.getState().connectEdge(c, a);
      expect(result).toBe(false);
    });
  });

  describe("undo/redo", () => {
    it("should undo last addNode", () => {
      usePipelineStore.getState().addNode("a");
      expect(usePipelineStore.getState().nodes).toHaveLength(1);
      usePipelineStore.getState().undo();
      expect(usePipelineStore.getState().nodes).toHaveLength(0);
    });

    it("should redo undone action", () => {
      usePipelineStore.getState().addNode("a");
      usePipelineStore.getState().undo();
      usePipelineStore.getState().redo();
      expect(usePipelineStore.getState().nodes).toHaveLength(1);
    });

    it("should limit undo stack to 50 steps", () => {
      for (let i = 0; i < 55; i++) usePipelineStore.getState().addNode(`p${i}`);
      expect(usePipelineStore.getState().undoStack.length).toBeLessThanOrEqual(50);
    });

    it("should do nothing when undo stack is empty", () => {
      const before = usePipelineStore.getState().nodes.length;
      usePipelineStore.getState().undo();
      expect(usePipelineStore.getState().nodes.length).toBe(before);
    });
  });

  describe("zoom/pan", () => {
    it("should clamp zoom between 0.1 and 5.0", () => {
      usePipelineStore.getState().setZoom(10);
      expect(usePipelineStore.getState().zoom).toBe(5.0);
      usePipelineStore.getState().setZoom(-1);
      expect(usePipelineStore.getState().zoom).toBe(0.1);
    });
  });

  describe("removeEdge", () => {
    it("should remove edge by id", () => {
      const a = usePipelineStore.getState().addNode("a");
      const b = usePipelineStore.getState().addNode("b");
      usePipelineStore.getState().connectEdge(a, b);
      const edgeId = usePipelineStore.getState().edges[0].id;
      usePipelineStore.getState().removeEdge(edgeId);
      expect(usePipelineStore.getState().edges).toHaveLength(0);
    });
  });
});
