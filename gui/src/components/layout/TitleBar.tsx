import { useAppStore, type AppMode } from "../../stores/useAppStore";
import { useBatchStore } from "../../stores/useBatchStore";
import "./TitleBar.css";

interface TitleBarProps {
  onOpenSettings: () => void;
}

export function TitleBar({ onOpenSettings }: TitleBarProps) {
  const mode = useAppStore((s) => s.mode);
  const setMode = useAppStore((s) => s.setMode);
  const queueLength = useBatchStore((s) => s.queue.length);

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="tb-logo">&#9670;</div>
      <span className="tb-title">Photopipeline</span>

      <div className="tb-mode-tabs">
        <button className={`tb-mode-tab ${mode === "edit" ? "active" : ""}`} onClick={() => setMode("edit")}>
          Pipeline Editor
        </button>
        <button className={`tb-mode-tab ${mode === "batch" ? "active" : ""}`} onClick={() => setMode("batch")}>
          Batch Processing
          {queueLength > 0 && <span className="tb-badge">{queueLength}</span>}
        </button>
      </div>

      <div style={{ flex: 1 }} />

      <button className="tb-btn" data-tauri-drag-region="false" title="Toggle theme (Dark/Light)">&#9684;</button>
      <button className="tb-btn" data-tauri-drag-region="false" title="Settings (Ctrl+,)" onClick={onOpenSettings}>&#9881;</button>
    </div>
  );
}
