import { useState } from "react";
import { useBatchStore } from "../../stores/useBatchStore";
import { BatchProgress } from "./BatchProgress";
import { BatchQueueList } from "./BatchQueueList";

export function BatchCenterPanel() {
  const batchState = useBatchStore((s) => s.batchState);
  const startBatch = useBatchStore((s) => s.startBatch);
  const pauseBatch = useBatchStore((s) => s.pauseBatch);
  const resumeBatch = useBatchStore((s) => s.resumeBatch);
  const stopBatch = useBatchStore((s) => s.stopBatch);
  const clearDone = useBatchStore((s) => s.clearDone);
  const queue = useBatchStore((s) => s.queue);
  const [showStopConfirm, setShowStopConfirm] = useState(false);

  const canStart = queue.length > 0 && (batchState === "idle" || batchState === "done");
  const canPause = batchState === "running";
  const canResume = batchState === "paused";
  const canStop = batchState === "running" || batchState === "paused";
  const canClear = (batchState === "done" || batchState === "idle") && queue.some((q) => q.status === "done" || q.status === "failed");
  const hasQueued = queue.some((q) => q.status === "queued");

  const handleStart = () => {
    if (batchState === "paused") { resumeBatch(); return; }
    startBatch();
  };

  const handleStop = () => {
    if (!showStopConfirm) { setShowStopConfirm(true); return; }
    stopBatch();
    setShowStopConfirm(false);
  };

  const handleClearDone = () => {
    clearDone();
  };

  return (
    <div className="batch-center">
      {/* Controls */}
      <div style={{ display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
        {canStart && (
          <button className="btn-primary-sm" style={{ height: 32, padding: "4px 16px", fontSize: "13px", fontWeight: 600 }}
            onClick={handleStart}>&#9654; Start Batch</button>
        )}
        {canResume && (
          <button className="btn-primary-sm" style={{ height: 32, padding: "4px 16px", fontSize: "13px", fontWeight: 600, background: "var(--warningFg)", borderColor: "var(--warningFg)" }}
            onClick={resumeBatch}>&#9654; Resume Batch</button>
        )}
        {canPause && (
          <button className="btn-subtle-sm" style={{ height: 32 }} onClick={pauseBatch}>&#9208; Pause</button>
        )}
        {canStop && (
          <button className="btn-subtle-sm" style={{ height: 32, color: showStopConfirm ? "var(--dangerFg)" : undefined, borderColor: showStopConfirm ? "var(--dangerFg)" : undefined }}
            onClick={handleStop}>{showStopConfirm ? "Confirm Stop?" : "⏹ Stop"}</button>
        )}
        {showStopConfirm && canStop && (
          <span style={{ fontSize: "10px", color: "var(--dangerFg)" }}>Running items will finish, queued items will be cancelled</span>
        )}
        <span style={{ flex: 1 }} />
        {canClear && (
          <button className="btn-subtle-sm" onClick={handleClearDone}>Clear Done</button>
        )}
      </div>

      {/* Progress */}
      <BatchProgress />

      {/* Queue */}
      {queue.length === 0 ? (
        <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", color: "var(--neutralFg4)", gap: "8px", textAlign: "center" }}>
          <div style={{ fontSize: "32px", opacity: 0.2 }}>&#128230;</div>
          <div style={{ fontSize: "12px" }}>No items in batch queue</div>
          <div style={{ fontSize: "10px", lineHeight: 1.5 }}>
            Switch to Pipeline Editor &rarr; select images &rarr;<br />click "To Batch" or right-click &rarr; "Send to Batch"
          </div>
        </div>
      ) : (
        <BatchQueueList />
      )}
    </div>
  );
}
