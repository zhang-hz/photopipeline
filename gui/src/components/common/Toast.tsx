import { useEffect, useState, useCallback } from "react";
import { create } from "zustand";
import "./Toast.css";

export interface ToastItem {
  id: string;
  message: string;
  type: "info" | "warning" | "error" | "success";
  duration?: number;
}

interface ToastState {
  toasts: ToastItem[];
  addToast: (message: string, type?: ToastItem["type"], duration?: number) => void;
  removeToast: (id: string) => void;
}

export const useToastStore = create<ToastState>((set) => ({
  toasts: [],
  addToast: (message, type = "info", duration = 4000) => {
    const id = `toast-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
    set((s) => ({ toasts: [...s.toasts, { id, message, type, duration }] }));
  },
  removeToast: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));

export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const removeToast = useToastStore((s) => s.removeToast);

  return (
    <div className="toast-container">
      {toasts.map((t) => (
        <ToastItemView key={t.id} item={t} onRemove={() => removeToast(t.id)} />
      ))}
    </div>
  );
}

function ToastItemView({ item, onRemove }: { item: ToastItem; onRemove: () => void }) {
  const [exiting, setExiting] = useState(false);

  useEffect(() => {
    if (!item.duration || item.duration <= 0) return;
    const timer = setTimeout(() => { setExiting(true); setTimeout(onRemove, 300); }, item.duration);
    return () => clearTimeout(timer);
  }, [item.duration, onRemove]);

  const handleClick = () => { setExiting(true); setTimeout(onRemove, 300); };

  const icon = item.type === "error" ? "✕" : item.type === "warning" ? "⚠" : item.type === "success" ? "✓" : "ℹ";

  return (
    <div className={`toast toast--${item.type} ${exiting ? "toast--exit" : ""}`} onClick={handleClick}>
      <span className="toast-icon">{icon}</span>
      <span className="toast-msg">{item.message}</span>
    </div>
  );
}
