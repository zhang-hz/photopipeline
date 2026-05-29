import { useRef, useState, useEffect, useCallback } from "react";
import { usePipelineStore } from "../../stores/usePipelineStore";
import { usePluginStore } from "../../stores/usePluginStore";
import { useFilmstripStore } from "../../stores/useFilmstripStore";
import { PluginBrowser } from "../panel/PluginBrowser";
import { DAGNode } from "../dag/DAGNode";
import { DAGEdge } from "../dag/DAGEdge";
import { MiniMap } from "../dag/MiniMap";
import { ContextMenu, type MenuItem } from "../common/ContextMenu";
import { useToastStore } from "../common/Toast";
import { autoLayout } from "../../utils/dagLayout";
import type { DAGNodeData } from "../../types/pipeline";

export function Content() {
  const nodes = usePipelineStore((s) => s.nodes);
  const edges = usePipelineStore((s) => s.edges);
  const selectedNodeId = usePipelineStore((s) => s.selectedNodeId);
  const selectedEdgeId = usePipelineStore((s) => s.selectedEdgeId as string | null);
  const zoom = usePipelineStore((s) => s.zoom);
  const panOffset = usePipelineStore((s) => s.panOffset);
  const executionState = usePipelineStore((s) => s.executionState);
  const isDirty = usePipelineStore((s) => s.isDirty);
  const addNode = usePipelineStore((s) => s.addNode);
  const removeNode = usePipelineStore((s) => s.removeNode);
  const moveNode = usePipelineStore((s) => s.moveNode);
  const connectEdge = usePipelineStore((s) => s.connectEdge);
  const removeEdge = usePipelineStore((s) => s.removeEdge);
  const setZoom = usePipelineStore((s) => s.setZoom);
  const setPan = usePipelineStore((s) => s.setPan);
  const selectNode = usePipelineStore((s) => s.selectNode);
  const undo = usePipelineStore((s) => s.undo);
  const redo = usePipelineStore((s) => s.redo);
  const newPipeline = usePipelineStore((s) => s.newPipeline);
  const plugins = usePluginStore((s) => s.plugins);
  const images = useFilmstripStore((s) => s.images);
  const selectedIndices = useFilmstripStore((s) => s.selectedIndices);

  const canvasRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState<{ nodeId: string; offsetX: number; offsetY: number } | null>(null);
  const [wireDrag, setWireDrag] = useState<{ fromNodeId: string; mouseX: number; mouseY: number } | null>(null);
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; items: MenuItem[] } | null>(null);
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState<{ x: number; y: number } | null>(null);
  const [dragOverCanvas, setDragOverCanvas] = useState(false);

  const getCanvasPos = (e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return { x: 0, y: 0 };
    return {
      x: (e.clientX - rect.left - panOffset.x) / zoom,
      y: (e.clientY - rect.top - panOffset.y) / zoom,
    };
  };

  // Center-on-mouse zoom
  const handleWheel = useCallback((e: React.WheelEvent) => {
    if (!e.ctrlKey) return;
    e.preventDefault();
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;
    const delta = e.deltaY > 0 ? -0.1 : 0.1;
    const newZoom = Math.max(0.1, Math.min(5.0, zoom + delta));
    const scale = newZoom / zoom;
    setPan({
      x: mouseX - (mouseX - panOffset.x) * scale,
      y: mouseY - (mouseY - panOffset.y) * scale,
    });
    setZoom(newZoom);
  }, [zoom, panOffset, setZoom, setPan]);

  const handleCanvasMouseDown = (e: React.MouseEvent) => {
    if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
      setIsPanning(true);
      setPanStart({ x: e.clientX - panOffset.x, y: e.clientY - panOffset.y });
    } else if (e.button === 0) {
      selectNode(null);
    }
  };

  const handleCanvasMouseMove = (e: React.MouseEvent) => {
    if (dragging) {
      const pos = getCanvasPos(e);
      moveNode(dragging.nodeId, { x: pos.x - dragging.offsetX, y: pos.y - dragging.offsetY });
    }
    if (wireDrag) {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) setWireDrag({ ...wireDrag, mouseX: e.clientX - rect.left, mouseY: e.clientY - rect.top });
    }
    if (isPanning && panStart) {
      setPan({ x: e.clientX - panStart.x, y: e.clientY - panStart.y });
    }
    e.preventDefault();
  };

  const handleCanvasMouseUp = () => {
    setDragging(null);
    setWireDrag(null);
    setIsPanning(false);
    setPanStart(null);
  };

  const handleNodeDragStart = (id: string, e: React.MouseEvent) => {
    if (executionState === "running") return; // Lock during execution
    const node = nodes.find((n) => n.id === id);
    if (!node) return;
    const pos = getCanvasPos(e);
    setDragging({ nodeId: id, offsetX: pos.x - node.position.x, offsetY: pos.y - node.position.y });
  };

  const handlePortDragStart = (_nodeId: string, _portType: string, e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (rect) setWireDrag({ fromNodeId: _nodeId, mouseX: e.clientX - rect.left, mouseY: e.clientY - rect.top });
  };

  const handlePortDrop = (toNodeId: string, _e: React.MouseEvent) => {
    if (wireDrag && wireDrag.fromNodeId !== toNodeId) {
      connectEdge(wireDrag.fromNodeId, toNodeId);
    }
    setWireDrag(null);
  };

  const handleCanvasDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOverCanvas(false);
    const pluginId = e.dataTransfer.getData("pluginId");
    if (pluginId) {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const x = (e.clientX - rect.left - panOffset.x) / zoom - 68;
        const y = (e.clientY - rect.top - panOffset.y) / zoom - 15;
        addNode(pluginId, { x, y });
      }
    }
  };

  // Save PipelineConfig JSON
  const handleSave = () => {
    const config = {
      name: "My Pipeline",
      version: "1.0",
      pipelines: [{
        nodes: nodes.map((n) => ({
          id: n.id, plugin: n.pluginId, label: n.label, enabled: n.enabled, params: n.params,
        })),
        edges: edges.map((e) => ({ from: e.fromNode, to: e.toNode })),
      }],
      images: images.filter((_, i) => selectedIndices.has(i)).map((img) => ({ path: img.path })),
    };
    const json = JSON.stringify(config, null, 2);
    // Use download API for demo
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url; a.download = "pipeline_config.json"; a.click();
    URL.revokeObjectURL(url);
  };

  // Load PipelineConfig JSON from file
  const handleLoad = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const config = JSON.parse(text);
        const pipeline = config.pipelines?.[0];
        if (!pipeline) { useToastStore.getState().addToast("Invalid PipelineConfig: no pipelines found", "error"); return; }

        // Restore nodes and edges
        const loadedNodes = (pipeline.nodes ?? []).map((n: any) => ({
          id: n.id, pluginId: n.plugin, label: n.label ?? n.id,
          enabled: n.enabled ?? true, params: n.params ?? {},
          position: { x: 100 + Math.random() * 400, y: 60 + Math.random() * 200 },
          inputs: [n.id + "_in"], outputs: [n.id + "_out"],
        }));
        const loadedEdges = (pipeline.edges ?? []).map((e: any) => ({
          id: `edge_${e.from}_${e.to}`, fromNode: e.from, toNode: e.to,
        }));

        usePipelineStore.setState({ nodes: loadedNodes, edges: loadedEdges, isDirty: false, undoStack: [], redoStack: [] });
        useToastStore.getState().addToast(`Loaded: ${loadedNodes.length} nodes, ${loadedEdges.length} edges`, "success");
      } catch (err) {
        useToastStore.getState().addToast(`Failed to parse PipelineConfig: ${err}`, "error");
      }
    };
    input.click();
  };

  // Context menu for nodes
  const getNodeCtxItems = (nodeId: string): MenuItem[] => {
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return [];
    return [
      { type: "header", label: node.label },
      { type: "separator" },
      { type: "item", label: "Copy", shortcut: "Ctrl+C" },
      { type: "item", label: "Duplicate", shortcut: "Ctrl+D", onClick: () => addNode(node.pluginId, { x: node.position.x + 50, y: node.position.y + 50 }) },
      { type: "separator" },
      { type: "item", label: node.enabled ? "Disable" : "Enable", onClick: () => {
        usePipelineStore.setState((s) => ({
          nodes: s.nodes.map((n) => n.id === nodeId ? { ...n, enabled: !n.enabled } : n),
          isDirty: true,
        }));
      }},
      { type: "separator" },
      { type: "item", label: "Delete", shortcut: "Del", danger: true, onClick: () => removeNode(nodeId) },
    ];
  };

  // Context menu for canvas
  const getCanvasCtxItems = (): MenuItem[] => [
    { type: "header", label: "Add Node" },
    { type: "separator" },
    ...plugins.map((p) => ({ type: "item" as const, label: p.name, onClick: () => addNode(p.id) })),
  ];

  // Get port position for edge drawing
  const getPortPos = (nodeId: string, portType: "input" | "output") => {
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return { x: 0, y: 0 };
    return {
      x: portType === "output" ? node.position.x + 130 : node.position.x,
      y: node.position.y + 25,
    };
  };

  // MiniMap click navigation
  const handleMiniMapClick = (_x: number, _y: number) => {
    // Center viewport on clicked position (simplified)
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    const cx = rect.width / 2, cy = rect.height / 2;
    setPan({ x: cx - _x * zoom, y: cy - _y * zoom });
  };

  return (
    <div className="content">
      {/* Header */}
      <div style={{ padding: "8px 12px", display: "flex", alignItems: "center", gap: "8px", fontSize: "10px", fontWeight: 600, textTransform: "uppercase", letterSpacing: "0.6px", color: "var(--neutralFg3)", borderBottom: "1px solid var(--neutralStroke1)" }}>
        <span style={{ width: 8, height: 8, borderRadius: "50%", background: executionState === "running" ? "var(--warningFg)" : "var(--successFg)", display: "inline-block", animation: executionState === "running" ? "pulse 1s infinite" : "none" }} />
        Pipeline Editor
        {isDirty && <span style={{ color: "var(--warningFg)", fontSize: "9px", fontWeight: 400 }}>unsaved</span>}
        <span style={{ fontSize: "9px", color: "var(--neutralFg4)", fontWeight: 400, marginLeft: "auto" }}>{nodes.length} nodes &middot; {Math.round(zoom * 100)}%</span>
      </div>

      {/* Toolbar */}
      <div style={{ padding: "8px 12px", display: "flex", gap: "4px", borderBottom: "1px solid var(--neutralStroke1)" }}>
        <button className="btn-subtle-sm" onClick={newPipeline}>New</button>
        <button className="btn-subtle-sm" onClick={handleSave} disabled={nodes.length === 0}>Save</button>
        <button className="btn-subtle-sm" onClick={handleLoad}>Load</button>
        <span style={{ width: 1, height: 20, background: "var(--neutralStroke1)", margin: "0 4px" }} />
        <button className="btn-subtle-sm" disabled={nodes.length === 0}>Validate</button>
        <span style={{ width: 1, height: 20, background: "var(--neutralStroke1)", margin: "0 4px" }} />
        <button className="btn-primary-sm" disabled={nodes.length === 0 || executionState === "running"}>&#9654; Run</button>
        <span style={{ flex: 1 }} />
        <button className="btn-subtle-sm" onClick={() => setZoom(zoom + 0.1)} disabled={zoom >= 5.0}>+</button>
        <span style={{ fontSize: "10px", color: "var(--neutralFg4)", width: 36, textAlign: "center" }}>{Math.round(zoom * 100)}%</span>
        <button className="btn-subtle-sm" onClick={() => setZoom(zoom - 0.1)} disabled={zoom <= 0.1}>&minus;</button>
        <button className="btn-subtle-sm" onClick={() => { setZoom(1.0); setPan({ x: 0, y: 0 }); }}>&#9974;</button>
        <button className="btn-subtle-sm" onClick={() => {
          const positions = autoLayout(nodes.map(n => ({ id: n.id })), edges.map(e => ({ from: e.fromNode, to: e.toNode })));
          positions.forEach((pos, id) => moveNode(id, pos));
        }} disabled={nodes.length === 0} title="Auto-layout nodes">&#9776;</button>
        <span style={{ width: 1, height: 20, background: "var(--neutralStroke1)", margin: "0 4px" }} />
        <button className="btn-subtle-sm" onClick={undo} disabled={usePipelineStore.getState().undoStack.length === 0} title="Undo (Ctrl+Z)">&#8634;</button>
        <button className="btn-subtle-sm" onClick={redo} disabled={usePipelineStore.getState().redoStack.length === 0} title="Redo (Ctrl+Y)">&#8635;</button>
      </div>

      {/* DAG Canvas */}
      <div
        ref={canvasRef}
        style={{
          flex: 1, margin: "8px", background: "var(--neutralBg1)", borderRadius: "var(--radiusLarge)",
          border: `1px solid ${dragOverCanvas ? "var(--brandFg1)" : "var(--neutralStroke1)"}`,
          minHeight: 300, position: "relative", overflow: "hidden",
          backgroundImage: `linear-gradient(rgba(255,255,255,0.015) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.015) 1px, transparent 1px)`,
          backgroundSize: `${32 * zoom}px ${32 * zoom}px`,
          backgroundPosition: `${panOffset.x}px ${panOffset.y}px`,
          cursor: isPanning ? "grabbing" : "default",
        }}
        onMouseDown={handleCanvasMouseDown}
        onMouseMove={handleCanvasMouseMove}
        onMouseUp={handleCanvasMouseUp}
        onMouseLeave={handleCanvasMouseUp}
        onWheel={handleWheel}
        onDragOver={(e) => { e.preventDefault(); setDragOverCanvas(true); }}
        onDragLeave={() => setDragOverCanvas(false)}
        onDrop={handleCanvasDrop}
        onContextMenu={(e) => {
          e.preventDefault();
          const dnode = (e.target as HTMLElement).closest(".dnode");
          if (dnode) {
            const nodeId = dnode.getAttribute("data-node-id") ?? "";
            setCtxMenu({ x: e.clientX, y: e.clientY, items: getNodeCtxItems(nodeId) });
          } else {
            setCtxMenu({ x: e.clientX, y: e.clientY, items: getCanvasCtxItems() });
          }
        }}
      >
        {/* Transform layer */}
        <div style={{ position: "absolute", inset: 0, transform: `scale(${zoom})`, transformOrigin: "0 0" }}>
          <div style={{ position: "absolute", left: panOffset.x / zoom, top: panOffset.y / zoom }}>
            {/* Edges */}
            <svg style={{ position: "absolute", inset: 0, pointerEvents: "none", overflow: "visible" }}>
              {edges.map((edge) => {
                const from = getPortPos(edge.fromNode, "output");
                const to = getPortPos(edge.toNode, "input");
                return (
                  <g key={edge.id}>
                    <path
                      d={`M ${from.x} ${from.y} C ${from.x + Math.abs(to.x - from.x) * 0.5} ${from.y}, ${to.x - Math.abs(to.x - from.x) * 0.5} ${to.y}, ${to.x} ${to.y}`}
                      fill="none" stroke="transparent" strokeWidth={12}
                      style={{ pointerEvents: "stroke", cursor: "pointer" }}
                      onClick={(e) => { e.stopPropagation(); }}
                      onDoubleClick={() => removeEdge(edge.id)}
                    />
                    <path
                      d={`M ${from.x} ${from.y} C ${from.x + Math.abs(to.x - from.x) * 0.5} ${from.y}, ${to.x - Math.abs(to.x - from.x) * 0.5} ${to.y}, ${to.x} ${to.y}`}
                      fill="none" stroke="var(--brandFg1)" strokeWidth={2} opacity={0.55}
                      style={{ pointerEvents: "none" }}
                    />
                  </g>
                );
              })}
              {/* Wire drag line */}
              {wireDrag && (() => {
                const fn = nodes.find((n) => n.id === wireDrag.fromNodeId);
                if (!fn) return null;
                const rect = canvasRef.current?.getBoundingClientRect();
                const from = { x: fn.position.x + 130, y: fn.position.y + 25 };
                const to = rect ? {
                  x: (wireDrag.mouseX - rect.left) / zoom - panOffset.x / zoom,
                  y: (wireDrag.mouseY - rect.top) / zoom - panOffset.y / zoom,
                } : { x: 0, y: 0 };
                return <line x1={from.x} y1={from.y} x2={to.x} y2={to.y} stroke="var(--brandFg1)" strokeWidth={2} strokeDasharray="6 3" opacity={0.6} />;
              })()}
            </svg>

            {/* Nodes */}
            {nodes.map((node) => (
              <div key={node.id} data-node-id={node.id} style={{ opacity: node.enabled ? 1 : 0.4 }}>
                <DAGNode data={node} selected={node.id === selectedNodeId} executing={false}
                  onDragStart={handleNodeDragStart} onPortDragStart={handlePortDragStart}
                  onPortDrop={handlePortDrop}
                  onContextMenu={(id, e) => setCtxMenu({ x: e.clientX, y: e.clientY, items: getNodeCtxItems(id) })} />
              </div>
            ))}
          </div>
        </div>

        {/* Drop hint */}
        {dragOverCanvas && (
          <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", pointerEvents: "none", zIndex: 15 }}>
            <div style={{ padding: "24px 48px", border: "2px dashed var(--brandFg1)", borderRadius: "var(--radiusLarge)", background: "rgba(71,158,245,0.06)", color: "var(--brandFg1)", fontSize: "14px", fontWeight: 600 }}>
              Drop plugin here
            </div>
          </div>
        )}

        {/* Empty state */}
        {nodes.length === 0 && (
          <div style={{ position: "absolute", top: "50%", left: "50%", transform: "translate(-50%,-50%)", textAlign: "center", color: "var(--neutralFg4)", fontSize: "12px", pointerEvents: "none" }}>
            <div style={{ fontSize: "40px", opacity: 0.2, marginBottom: "8px" }}>&#9678;</div>
            <div>Drop a plugin here or right-click to add</div>
          </div>
        )}

        {/* MiniMap */}
        <MiniMap nodes={nodes} edges={edges} canvasRef={canvasRef} onNavigate={handleMiniMapClick} />
      </div>

      <PluginBrowser onAddNode={(pluginId) => addNode(pluginId)} />

      <ContextMenu isOpen={ctxMenu !== null} x={ctxMenu?.x ?? 0} y={ctxMenu?.y ?? 0}
        items={ctxMenu?.items ?? []} onClose={() => setCtxMenu(null)} />
    </div>
  );
}
