import { useState } from "react";
import { usePipelineStore } from "../../stores/usePipelineStore";

interface RemoveButtonProps {
  nodeId: string;
  nodeLabel: string;
}

export function RemoveButton({ nodeId, nodeLabel }: RemoveButtonProps) {
  const removeNode = usePipelineStore((s) => s.removeNode);
  const [showConfirm, setShowConfirm] = useState(false);

  if (showConfirm) {
    return (
      <div style={{ padding: "0 16px 16px", display: "flex", flexDirection: "column", gap: "8px" }}>
        <div style={{ fontSize: "var(--fontSizeBody1)", color: "var(--neutralFg2)", textAlign: "center" }}>
          Remove "{nodeLabel}" from pipeline?
        </div>
        <div style={{ fontSize: "10px", color: "var(--neutralFg4)", textAlign: "center" }}>
          Connected edges will also be deleted.
        </div>
        <div style={{ display: "flex", gap: "8px" }}>
          <button className="btn-subtle-sm" style={{ flex: 1 }} onClick={() => setShowConfirm(false)}>Cancel</button>
          <button className="btn-subtle-sm" style={{ flex: 1, borderColor: "var(--dangerFg)", color: "var(--dangerFg)" }}
            onClick={() => { removeNode(nodeId); setShowConfirm(false); }}>Remove</button>
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: "0 16px 16px" }}>
      <button
        style={{
          width: "100%", height: 30, background: "transparent",
          border: "1px solid var(--dangerFg)", borderRadius: "var(--radiusMedium)",
          color: "var(--dangerFg)", fontSize: "var(--fontSizeBody1)", cursor: "pointer",
        }}
        onMouseEnter={(e) => { e.currentTarget.style.background = "rgba(209,52,71,0.08)"; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
        onClick={() => setShowConfirm(true)}
      >
        &#128465; Remove from Pipeline
      </button>
    </div>
  );
}
