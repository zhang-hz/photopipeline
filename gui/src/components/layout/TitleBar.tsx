import { useAppStore, type AppMode } from "../../stores/useAppStore";
import { useBatchStore } from "../../stores/useBatchStore";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./TitleBar.css";

interface TitleBarProps {
  onOpenSettings: () => void;
}

export function TitleBar({ onOpenSettings }: TitleBarProps) {
  const mode = useAppStore((s) => s.mode);
  const theme = useAppStore((s) => s.theme);
  const setMode = useAppStore((s) => s.setMode);
  const setTheme = useAppStore((s) => s.setTheme);
  const queueLength = useBatchStore((s) => s.queue.length);

  const switchMode = (m: AppMode) => setMode(m);
  const toggleTheme = () => setTheme(theme === "dark" ? "light" : "dark");

  const appWindow = getCurrentWindow();

  const handleMinimize = () => appWindow.minimize();
  const handleMaximize = () => appWindow.toggleMaximize();
  const handleClose = () => appWindow.close();

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="tb-logo">&#9670;</div>
      <span className="tb-title">Photopipeline</span>

      <div className="tb-mode-tabs">
        <button className={`tb-mode-tab ${mode === "edit" ? "active" : ""}`} onClick={() => switchMode("edit")}>
          管线编辑器
        </button>
        <button className={`tb-mode-tab ${mode === "batch" ? "active" : ""}`} onClick={() => switchMode("batch")}>
          批量处理
          {queueLength > 0 && <span className="tb-badge">{queueLength}</span>}
        </button>
      </div>

      <div style={{ flex: 1 }} />

      <button className="tb-btn" data-tauri-drag-region="false" title={theme === "dark" ? "切换到明亮模式" : "切换到暗黑模式"} onClick={toggleTheme}>
        {theme === "dark" ? "☀" : "◐"}
      </button>
      <button className="tb-btn" data-tauri-drag-region="false" title="设置 (Ctrl+,)" onClick={onOpenSettings}>&#9881;</button>

      <span style={{ width: 1, height: 20, background: "var(--neutralStroke1)", margin: "0 4px" }} />

      <button className="tb-btn" data-tauri-drag-region="false" title="最小化" onClick={handleMinimize}>&#9472;</button>
      <button className="tb-btn" data-tauri-drag-region="false" title="最大化" onClick={handleMaximize}>&#9744;</button>
      <button className="tb-btn" data-tauri-drag-region="false" title="关闭" onClick={handleClose} style={{ color: "var(--dangerFg)" }}>&#10005;</button>
    </div>
  );
}
