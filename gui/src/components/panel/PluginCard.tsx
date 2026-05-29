import type { PluginEntry } from "../../types/plugin";
import "./PluginCard.css";

const PLUGIN_COLORS: Record<string, string> = {
  "Input":     "var(--plugin-raw_input)",
  "Transform": "var(--plugin-transform)",
  "Color":     "var(--plugin-colorspace)",
  "Correct":   "var(--plugin-lens_correct)",
  "Enhance":   "var(--plugin-ai_denoise)",
  "Metadata":  "var(--plugin-exif_rw)",
  "Export":    "var(--plugin-heif_encoder)",
};

interface PluginCardProps {
  plugin: PluginEntry;
  isSelected?: boolean;
  onDoubleClick?: (pluginId: string) => void;
}

export function PluginCard({ plugin, isSelected, onDoubleClick }: PluginCardProps) {
  const color = PLUGIN_COLORS[plugin.category] ?? "var(--brandFg1)";

  return (
    <div
      className={`pcard ${isSelected ? "pcard--sel" : ""}`}
      onDoubleClick={() => onDoubleClick?.(plugin.id)}
      draggable
      onDragStart={(e) => { e.dataTransfer.setData("pluginId", plugin.id); }}
      title={plugin.description}
    >
      <div className="pcard-dot" style={{ color }}>&#9679;</div>
      <div className="pcard-name">{plugin.name}</div>
      <div className="pcard-cat">{plugin.category}</div>
    </div>
  );
}
