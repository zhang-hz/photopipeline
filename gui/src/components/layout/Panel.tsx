import { usePipelineStore } from "../../stores/usePipelineStore";
import { ContextBar } from "../panel/ContextBar";
import { PluginHeader } from "../panel/PluginHeader";
import { ControlPanel } from "../panel/ControlPanel";

export function Panel() {
  const selectedNodeId = usePipelineStore((s) => s.selectedNodeId);
  const nodes = usePipelineStore((s) => s.nodes);
  const selectedNode = nodes.find((n) => n.id === selectedNodeId);

  return (
    <div className="panel">
      <ContextBar />

      {selectedNode ? (
        <>
          <PluginHeader pluginId={selectedNode.pluginId} />
          <ControlPanel
            nodeId={selectedNode.id}
            nodeLabel={selectedNode.label}
            pluginId={selectedNode.pluginId}
          />
        </>
      ) : (
        <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", color: "var(--neutralFg4)", gap: "8px", padding: "0 24px", textAlign: "center" }}>
          <div style={{ fontSize: "32px", opacity: 0.2 }}>&#9881;</div>
          <div style={{ fontSize: "var(--fontSizeBody1)" }}>Select a node to edit parameters</div>
          <div style={{ fontSize: "10px", lineHeight: 1.5 }}>
            Click a node in the DAG canvas<br />or add one from the plugin browser below
          </div>
        </div>
      )}
    </div>
  );
}
