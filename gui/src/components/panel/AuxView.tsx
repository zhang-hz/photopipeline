import { useRef, useEffect } from "react";

interface AuxViewProps { type: string; }

const TITLES: Record<string, string> = {
  histogram: "Histogram", waveform: "Waveform", vectorscope: "Vectorscope",
  gamut_diagram: "Gamut Diagram", map: "Map", status_text: "Status",
  progress_bar: "Progress", focus_peaking: "Focus Peaking",
};

export function AuxView({ type }: AuxViewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);

    if (type === "histogram") drawHistogram(ctx, W, H);
    else if (type === "waveform") drawWaveform(ctx, W, H);
    else if (type === "vectorscope") drawVectorscope(ctx, W, H);
    else if (type === "gamut_diagram") drawGamutDiagram(ctx, W, H);
  }, [type]);

  const title = TITLES[type] ?? type;
  const isCanvas = ["histogram", "waveform", "vectorscope", "gamut_diagram", "focus_peaking"].includes(type);

  return (
    <div style={{ background: "var(--neutralBg1)", borderRadius: "var(--radiusLarge)", border: "1px dashed var(--neutralStroke1)", padding: "8px 12px" }}>
      <div style={{ fontSize: "10px", fontWeight: 600, color: "var(--neutralFg3)", textTransform: "uppercase", marginBottom: "6px" }}>{title}</div>
      {isCanvas ? (
        <canvas ref={canvasRef} width={380} height={type === "vectorscope" ? 120 : type === "gamut_diagram" ? 120 : 60} style={{ width: "100%", display: "block" }} />
      ) : (
        <div style={{ fontSize: "11px", color: "var(--neutralFg4)", textAlign: "center", padding: "8px 0" }}>
          {type === "status_text" ? "Lens detected: Sony FE 24-70mm F2.8 GM II" :
           type === "progress_bar" ? "AI inference: 78% complete" :
           type === "map" ? "Map view (requires GPS data)" : `${type} view`}
        </div>
      )}
    </div>
  );
}

function drawHistogram(ctx: CanvasRenderingContext2D, W: number, H: number) {
  const bins = 12;
  const r = [2,5,12,28,45,62,58,41,24,10,4,1], g = [1,3,8,20,38,55,60,48,30,14,5,2], b = [0,2,5,14,28,42,50,38,22,9,3,1], l = [1,4,10,24,40,56,58,44,27,12,5,1];
  const maxV = Math.max(...r, ...g, ...b);
  const barW = (W - bins) / bins;
  [r, g, b, l].forEach((ch, ci) => {
    const alpha = ci === 3 ? 0.3 : 0.7, col = ci === 0 ? "239,68,68" : ci === 1 ? "84,176,84" : ci === 2 ? "71,158,245" : "200,200,200";
    ch.forEach((v, i) => {
      ctx.fillStyle = `rgba(${col},${alpha})`;
      ctx.fillRect(i * (barW + 1), H - (v / maxV) * (H - 4), barW, (v / maxV) * (H - 4));
    });
  });
}

function drawWaveform(ctx: CanvasRenderingContext2D, W: number, H: number) {
  // Synthetic waveform — luminance across columns
  for (let x = 0; x < W; x++) {
    const px = x / W;
    const val = 0.5 + 0.4 * Math.sin(px * 20) * Math.cos(px * 3) + 0.1 * Math.sin(px * 47);
    const y = H * (1 - val);
    ctx.strokeStyle = `rgba(71,158,245,${0.3 + val * 0.5})`;
    ctx.beginPath(); ctx.moveTo(x, y - 2); ctx.lineTo(x, y + 2); ctx.stroke();
  }
}

function drawVectorscope(ctx: CanvasRenderingContext2D, W: number, H: number) {
  const cx = W / 2, cy = H / 2, r = Math.min(W, H) / 2 - 4;
  // Background grid
  ctx.strokeStyle = "rgba(255,255,255,0.05)"; ctx.lineWidth = 0.5;
  ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2); ctx.stroke();
  ctx.beginPath(); ctx.moveTo(cx - r, cy); ctx.lineTo(cx + r, cy); ctx.stroke();
  ctx.beginPath(); ctx.moveTo(cx, cy - r); ctx.lineTo(cx, cy + r); ctx.stroke();
  // Targets
  const targets = [{ u: 0.3, v: 0.5 }, { u: -0.2, v: -0.3 }, { u: 0.1, v: -0.2 }, { u: -0.15, v: 0.1 }, { u: 0.25, v: -0.1 }, { u: -0.3, v: 0.2 }];
  ctx.fillStyle = "rgba(71,158,245,0.4)";
  targets.forEach(({ u, v }) => { ctx.beginPath(); ctx.arc(cx + u * r, cy + v * r, 2, 0, Math.PI * 2); ctx.fill(); });
}

function drawGamutDiagram(ctx: CanvasRenderingContext2D, W: number, H: number) {
  // Simplified CIE 1931 outline (horseshoe shape approximated as polygon)
  const cx = W / 2, cy = H / 2, r = Math.min(W, H) / 2 - 4;
  const pts: [number, number][] = [
    [0.64, 0.33], [0.30, 0.60], [0.15, 0.06], [0.30, 0.60], [0.64, 0.33],
  ];
  ctx.strokeStyle = "rgba(71,158,245,0.6)"; ctx.lineWidth = 1.5; ctx.beginPath();
  pts.forEach(([x, y], i) => {
    const px = cx + (x - 0.4) * r * 2, py = cy + (y - 0.3) * r * 2;
    i === 0 ? ctx.moveTo(px, py) : ctx.lineTo(px, py);
  });
  ctx.closePath(); ctx.stroke();
  // Fill with light blue
  ctx.fillStyle = "rgba(71,158,245,0.05)"; ctx.fill();
  // Pixel distribution
  for (let i = 0; i < 50; i++) {
    const px = cx + (Math.random() - 0.5) * r * 1.5, py = cy + (Math.random() - 0.5) * r * 1.5;
    ctx.fillStyle = `rgba(71,158,245,${0.1 + Math.random() * 0.3})`;
    ctx.fillRect(px, py, 1.5, 1.5);
  }
}
