import { useRef, useEffect } from "react";
import type { DAGNodeData, DAGEdgeData } from "../../types/pipeline";
import { PLUGIN_COLORS } from "../../data/mockPlugins";

interface MiniMapProps {
  nodes: DAGNodeData[];
  edges: DAGEdgeData[];
  canvasRef: React.RefObject<HTMLDivElement | null>;
  onNavigate?: (x: number, y: number) => void;
}

export function MiniMap({ nodes, edges, canvasRef, onNavigate }: MiniMapProps) {
  const miniRef = useRef<HTMLCanvasElement>(null);
  const boundsRef = useRef({ minX: 0, maxX: 1, minY: 0, maxY: 1, scale: 1, offsetX: 0, offsetY: 0, tx: (v: number) => 0, ty: (v: number) => 0 });

  useEffect(() => {
    const canvas = miniRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const W = 128, H = 84;
    canvas.width = W;
    canvas.height = H;

    ctx.fillStyle = "rgba(0,0,0,0.55)";
    ctx.fillRect(0, 0, W, H);
    ctx.strokeStyle = "#383838";
    ctx.lineWidth = 1;
    ctx.strokeRect(0.5, 0.5, W - 1, H - 1);

    if (nodes.length === 0) return;

    const xs = nodes.map((n) => n.position.x);
    const ys = nodes.map((n) => n.position.y);
    const minX = Math.min(...xs) - 200;
    const maxX = Math.max(...xs) + 300;
    const minY = Math.min(...ys) - 100;
    const maxY = Math.max(...ys) + 150;
    const s = Math.min(W / (maxX - minX || 1), H / (maxY - minY || 1)) * 0.85;
    const ox = (W - (maxX - minX) * s) / 2;
    const oy = (H - (maxY - minY) * s) / 2;
    const tx = (vx: number) => (vx - minX) * s + ox;
    const ty = (vy: number) => (vy - minY) * s + oy;

    boundsRef.current = { minX, maxX, minY, maxY, scale: s, offsetX: ox, offsetY: oy, tx, ty };

    // Edges
    ctx.strokeStyle = "rgba(71,158,245,0.4)";
    ctx.lineWidth = 1;
    for (const e of edges) {
      const fn = nodes.find((n) => n.id === e.fromNode);
      const tn = nodes.find((n) => n.id === e.toNode);
      if (fn && tn) {
        ctx.beginPath();
        ctx.moveTo(tx(fn.position.x + 65), ty(fn.position.y + 12));
        ctx.lineTo(tx(tn.position.x), ty(tn.position.y + 12));
        ctx.stroke();
      }
    }

    // Nodes
    for (const n of nodes) {
      const shortId = n.pluginId.replace("photopipeline.plugins.", "");
      ctx.fillStyle = PLUGIN_COLORS[shortId] ?? "#479ef5";
      ctx.fillRect(tx(n.position.x), ty(n.position.y), Math.max(50 * s, 4), Math.max(25 * s, 3));
    }

    // Viewport indicator
    if (canvasRef.current) {
      const rect = canvasRef.current.getBoundingClientRect();
      if (rect) {
        const store = { zoom: 1, panOffset: { x: 0, y: 0 } };
        try {
          const state = (window as any).__pipelineStoreSnapshot;
          if (state) { store.zoom = state.zoom; store.panOffset = state.panOffset; }
        } catch {}
        ctx.strokeStyle = "rgba(71,158,245,0.6)";
        ctx.lineWidth = 1;
        const vpX = tx(-store.panOffset.x / store.zoom);
        const vpY = ty(-store.panOffset.y / store.zoom);
        const vpW = rect.width / store.zoom * s;
        const vpH = rect.height / store.zoom * s;
        ctx.strokeRect(vpX, vpY, vpW, vpH);
      }
    }
  }, [nodes, edges, canvasRef]);

  return (
    <canvas
      ref={miniRef}
      style={{
        position: "absolute", bottom: 12, right: 12, zIndex: 5,
        borderRadius: "var(--radiusMedium)", border: "1px solid var(--neutralStroke1)",
        cursor: "pointer",
      }}
      onClick={(e) => {
        if (!onNavigate || nodes.length === 0) return;
        const rect = miniRef.current?.getBoundingClientRect();
        if (!rect) return;
        const cx = (e.clientX - rect.left);
        const cy = (e.clientY - rect.top);
        const { minX, minY, scale, offsetX, offsetY } = boundsRef.current;
        const worldX = (cx - offsetX) / scale + minX;
        const worldY = (cy - offsetY) / scale + minY;
        onNavigate(worldX, worldY);
      }}
      title="MiniMap — click to navigate"
    />
  );
}
