import { usePipelineStore } from "../../stores/usePipelineStore";
import type { DAGNodeData } from "../../types/pipeline";
import { PLUGIN_COLORS } from "../../data/mockPlugins";
import "./DAGNode.css";

interface DAGNodeProps {
  data: DAGNodeData;
  selected: boolean;
  executing: boolean;
  onDragStart: (id: string, e: React.MouseEvent) => void;
  onPortDragStart: (nodeId: string, portType: "input" | "output", e: React.MouseEvent) => void;
  onPortDrop: (nodeId: string, e: React.MouseEvent) => void;
  onContextMenu: (id: string, e: React.MouseEvent) => void;
}

export function DAGNode({ data, selected, executing, onDragStart, onPortDragStart, onPortDrop, onContextMenu }: DAGNodeProps) {
  const shortPluginId = data.pluginId.replace("photopipeline.plugins.", "");
  const color = PLUGIN_COLORS[shortPluginId] ?? "var(--brandFg1)";

  return (
    <div
      className={`dnode ${selected ? "dnode--sel" : ""} ${executing ? "dnode--exec" : ""}`}
      style={{
        position: "absolute", left: data.position.x, top: data.position.y,
        borderColor: selected ? "var(--brandFg1)" : color + "44",
      }}
      onMouseDown={(e) => { if (e.button === 0) onDragStart(data.id, e); }}
      onClick={(e) => { e.stopPropagation(); usePipelineStore.getState().selectNode(data.id); }}
      onContextMenu={(e) => { e.preventDefault(); onContextMenu(data.id, e); }}
    >
      {/* Input port */}
      {data.inputs.map((pid) => (
        <div key={pid} className="dport dport--i"
          onMouseDown={(e) => { e.stopPropagation(); }}
          onMouseUp={(e) => { e.stopPropagation(); onPortDrop(data.id, e); }}
        />
      ))}

      <div className="dnode-name">{data.label}</div>
      <div className="dnode-cat" style={{ color: color }}>{shortPluginId}</div>

      {/* Output port */}
      {data.outputs.map((pid) => (
        <div key={pid} className="dport dport--o"
          onMouseDown={(e) => { e.stopPropagation(); onPortDragStart(data.id, "output", e); }}
        />
      ))}
    </div>
  );
}
