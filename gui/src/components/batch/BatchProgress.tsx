import { useBatchStore } from "../../stores/useBatchStore";

export function BatchProgress() {
  const progress = useBatchStore((s) => s.progress);
  const batchState = useBatchStore((s) => s.batchState);
  const queue = useBatchStore((s) => s.queue);
  const processingItem = queue.find((q) => q.status === "processing");

  if (!progress) return null;

  const { total, done, failed, elapsedSecs, etaSecs, speedPerMin } = progress;
  const pct = total > 0 ? ((done + failed) / total) * 100 : 0;

  const formatTime = (secs: number) => {
    if (secs <= 0 || !isFinite(secs)) return "--:--";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  };

  return (
    <div style={{ padding: "8px 0" }}>
      <div style={{ height: 6, background: "var(--neutralStroke1)", borderRadius: 3, overflow: "hidden", marginBottom: "8px" }}>
        <div style={{ height: "100%", background: batchState === "paused" ? "var(--warningFg)" : "var(--brandFg1)", width: `${pct}%`, transition: "width 0.3s ease", borderRadius: 3 }} />
      </div>

      <div style={{ display: "flex", gap: "16px", fontSize: "10px", color: "var(--neutralFg4)", flexWrap: "wrap", alignItems: "center" }}>
        <span style={{ fontWeight: 600, color: "var(--neutralFg2)", minWidth: 32 }}>{Math.round(pct)}%</span>
        <span style={{ color: "var(--successFg)" }}>{done} done</span>
        {failed > 0 && <span style={{ color: "var(--dangerFg)" }}>{failed} failed</span>}
        <span>{total} total</span>
        <span style={{ color: "var(--neutralFg2)" }}>&#9202; {formatTime(elapsedSecs)}</span>
        {pct > 5 && pct < 100 && <span style={{ color: "var(--neutralFg3)" }}>~{formatTime(etaSecs)} remaining</span>}
        {speedPerMin > 0 && <span>{speedPerMin} img/min</span>}
        {batchState === "paused" && <span style={{ color: "var(--warningFg)", fontWeight: 600 }}>PAUSED</span>}
        {batchState === "running" && <span style={{ color: "var(--brandFg1)", animation: "pulse 1s infinite" }}>Processing...</span>}
        {processingItem && batchState === "running" && (
          <span style={{ color: "var(--neutralFg3)", fontStyle: "italic" }}>
            Current: {processingItem.image.filename}
          </span>
        )}
      </div>
    </div>
  );
}
