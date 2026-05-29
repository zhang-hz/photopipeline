import { useAppStore } from "../../stores/useAppStore";
import { useBatchStore } from "../../stores/useBatchStore";
import { usePipelineStore } from "../../stores/usePipelineStore";

export function StatusBar() {
  const mode = useAppStore((s) => s.mode);
  const isBackendConnected = useAppStore((s) => s.isBackendConnected);
  const queue = useBatchStore((s) => s.queue);
  const executionState = usePipelineStore((s) => s.executionState);

  const total = queue.length;
  const done = queue.filter((q) => q.status === "done").length;
  const failed = queue.filter((q) => q.status === "failed").length;
  const pct = total > 0 ? ((done + failed) / total) * 100 : 0;

  return (
    <div className="statusbar">
      {mode === "batch" && total > 0 ? (
        <>
          <span style={{ color: "var(--neutralFg2)" }}>&#9654; Batch: {done + failed}/{total}</span>
          <div style={{ flex: 1, maxWidth: 180, height: 4, background: "var(--neutralStroke1)", borderRadius: 2, overflow: "hidden" }}>
            <div style={{ height: "100%", background: "var(--brandFg1)", width: `${pct}%`, transition: "width 0.3s ease" }} />
          </div>
          <span style={{ fontSize: "10px", color: "var(--neutralFg2)" }}>{done} done</span>
          {failed > 0 && <span style={{ fontSize: "10px", color: "var(--dangerFg)" }}>{failed} failed</span>}
        </>
      ) : (
        <span style={{ color: "var(--neutralFg2)" }}>
          &#9654; {executionState === "running" ? "Running..." : "Ready"}
        </span>
      )}

      <span style={{ flex: 1 }} />

      <span style={{ fontSize: "10px", color: "var(--neutralFg4)" }}>Mem: --</span>
      <span style={{ color: "var(--neutralFg4)" }}>|</span>
      <span style={{ fontSize: "10px", color: "var(--neutralFg4)" }}>GPU: --</span>
      <span style={{ color: "var(--neutralFg4)" }}>|</span>
      <span style={{ width: 8, height: 8, borderRadius: "50%", background: isBackendConnected ? "var(--successFg)" : "var(--dangerFg)", display: "inline-block" }} />
      <span style={{ fontSize: "10px", color: isBackendConnected ? "var(--neutralFg2)" : "var(--dangerFg)" }}>
        {isBackendConnected ? "Connected" : "Disconnected"}
      </span>
    </div>
  );
}
