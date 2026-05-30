import type { PluginEntry } from "../../types/plugin";
import { usePipelineStore } from "../../stores/usePipelineStore";
import "./PluginCard.css";

const PLUGIN_COLORS: Record<string, string> = {
  "Input": "var(--plugin-raw_input)", "Transform": "var(--plugin-transform)",
  "Color": "var(--plugin-colorspace)", "Enhance": "var(--plugin-ai_denoise)",
  "Metadata": "var(--plugin-exif_rw)", "Export": "var(--plugin-heif_encoder)",
};

interface PluginCardProps {
  plugin: PluginEntry;
  isSelected?: boolean;
  onDoubleClick?: (pluginId: string) => void;
}

export function PluginCard({ plugin, isSelected, onDoubleClick }: PluginCardProps) {
  const color = PLUGIN_COLORS[plugin.category] ?? "var(--brandFg1)";

  const handleDragStart = (e: React.DragEvent) => {
    e.dataTransfer.setData("text/plain", plugin.id);
    e.dataTransfer.effectAllowed = "copy";
    // Also set via store for Tauri WebView compatibility
    usePipelineStore.setState({ _draggedPluginId: plugin.id });
  };

  const handleDragEnd = () => {
    usePipelineStore.setState({ _draggedPluginId: null });
  };

  return (
    <div
      className={`pcard ${isSelected ? "pcard--sel" : ""}`}
      onDoubleClick={() => onDoubleClick?.(plugin.id)}
      draggable
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      title={plugin.description}
    >
      <div className="pcard-dot" style={{ color }}>&#9679;</div>
      <div className="pcard-name">{plugin.name}</div>
      <div className="pcard-cat">{plugin.category}</div>
    </div>
  );
}
