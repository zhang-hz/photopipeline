// Persist window position and size via localStorage.
// In production, this would use Tauri window API for precise control.

const KEY = "photopipeline-window-state";

interface WindowState { x?: number; y?: number; width?: number; height?: number; maximized?: boolean; }

export function loadWindowState(): WindowState | null {
  try {
    const raw = localStorage.getItem(KEY);
    return raw ? JSON.parse(raw) : null;
  } catch { return null; }
}

export function saveWindowState(state: WindowState): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(state));
  } catch { /* silently ignore quota errors */ }
}

// Restore window size on mount (CSS vars based)
export function applyWindowState(): void {
  const state = loadWindowState();
  if (!state) return;
  // Apply size to the #app-root if we can
  const root = document.getElementById("app-root");
  if (root && state.width && state.height) {
    root.style.width = `${state.width}px`;
    root.style.height = `${state.height}px`;
  }
}
