import { useEffect } from "react";

export interface ShortcutDef {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: () => void;
  scope?: "global" | "filmstrip" | "dag" | "panel";
}

// Detect platform for shortcut display
const isMac = typeof navigator !== "undefined" && navigator.platform?.toLowerCase().includes("mac");

export function modKey(): string { return isMac ? "⌘" : "Ctrl"; }
export function formatShortcut(key: string, ctrl?: boolean, shift?: boolean, alt?: boolean): string {
  const parts: string[] = [];
  if (ctrl) parts.push(modKey());
  if (alt) parts.push(isMac ? "⌥" : "Alt");
  if (shift) parts.push(isMac ? "⇧" : "Shift");
  parts.push(key.length === 1 ? key.toUpperCase() : key);
  return parts.join(isMac ? "" : "+");
}

export function useKeyboard(shortcuts: ShortcutDef[]): void {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      for (const s of shortcuts) {
        if (
          e.key.toLowerCase() === s.key.toLowerCase() &&
          !!s.ctrl === (e.ctrlKey || e.metaKey) &&
          !!s.shift === e.shiftKey &&
          !!s.alt === e.altKey
        ) {
          e.preventDefault();
          s.handler();
          return;
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [shortcuts]);
}
