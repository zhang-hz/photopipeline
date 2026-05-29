import { useEffect, useRef } from "react";
import "./ContextMenu.css";

export interface MenuItem {
  type: "item" | "separator" | "header";
  label?: string;
  shortcut?: string;
  danger?: boolean;
  disabled?: boolean;
  onClick?: () => void;
  children?: MenuItem[];
}

interface ContextMenuProps {
  isOpen: boolean;
  x: number;
  y: number;
  items: MenuItem[];
  onClose: () => void;
}

export function ContextMenu({ isOpen, x, y, items, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const keyHandler = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    document.addEventListener("mousedown", handler);
    document.addEventListener("keydown", keyHandler);
    return () => {
      document.removeEventListener("mousedown", handler);
      document.removeEventListener("keydown", keyHandler);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div ref={ref} className="ctx-menu" style={{ left: x, top: y }}>
      {items.map((item, i) => {
        if (item.type === "separator") return <div key={i} className="ctx-sep" />;
        if (item.type === "header") return <div key={i} className="ctx-header">{item.label}</div>;
        if (item.children) {
          return (
            <div key={i} className="ctx-item ctx-has-sub">
              <span>{item.label}</span>
              <span className="ctx-sub-arrow">&#9654;</span>
              <div className="ctx-submenu">
                {item.children.map((child, j) => (
                  <div
                    key={j}
                    className={`ctx-item ${child.danger ? "ctx-danger" : ""} ${child.disabled ? "ctx-disabled" : ""}`}
                    onClick={() => { if (!child.disabled) { child.onClick?.(); onClose(); } }}
                  >
                    <span>{child.label}</span>
                    {child.shortcut && <span className="ctx-shortcut">{child.shortcut}</span>}
                  </div>
                ))}
              </div>
            </div>
          );
        }
        return (
          <div
            key={i}
            className={`ctx-item ${item.danger ? "ctx-danger" : ""} ${item.disabled ? "ctx-disabled" : ""}`}
            onClick={() => { if (!item.disabled) { item.onClick?.(); onClose(); } }}
          >
            <span>{item.label}</span>
            {item.shortcut && <span className="ctx-shortcut">{item.shortcut}</span>}
          </div>
        );
      })}
    </div>
  );
}
