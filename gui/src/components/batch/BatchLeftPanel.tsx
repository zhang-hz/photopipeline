import { usePipelineStore } from "../../stores/usePipelineStore";
import { useBatchStore } from "../../stores/useBatchStore";
import { useFilmstripStore } from "../../stores/useFilmstripStore";
import { PLUGIN_COLORS } from "../../data/mockPlugins";

export function BatchLeftPanel() {
  const nodes = usePipelineStore((s) => s.nodes);
  const edges = usePipelineStore((s) => s.edges);
  const outputSettings = useBatchStore((s) => s.outputSettings);
  const groups = useFilmstripStore((s) => s.groups);

  // Sort nodes by topological order from edges
  const sortedNodeIds = (() => {
    const result: string[] = [];
    const visited = new Set<string>();
    const visit = (id: string) => {
      if (visited.has(id)) return;
      visited.add(id);
      const incoming = edges.filter((e) => e.toNode === id);
      incoming.forEach((e) => visit(e.fromNode));
      result.push(id);
    };
    nodes.forEach((n) => visit(n.id));
    return result;
  })();

  return (
    <div className="batch-left">
      <div style={{ fontSize: "10px", fontWeight: 600, color: "var(--neutralFg3)", textTransform: "uppercase", letterSpacing: "0.6px" }}>
        Pipeline Summary
      </div>

      {/* Nodes chain */}
      <div style={{
        background: "var(--neutralBg2)", border: "1px solid var(--neutralStroke1)",
        borderRadius: "6px", padding: "8px",
      }}>
        <div style={{ fontSize: "10px", color: "var(--neutralFg4)", marginBottom: "4px" }}>Nodes</div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "4px", alignItems: "center" }}>
          {sortedNodeIds.map((nid, i) => {
            const node = nodes.find((n) => n.id === nid);
            if (!node) return null;
            const shortId = node.pluginId.replace("photopipeline.plugins.", "");
            return (
              <span key={nid} style={{ display: "flex", alignItems: "center", gap: "4px" }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: PLUGIN_COLORS[shortId] ?? "var(--brandFg1)", display: "inline-block" }} />
                <span style={{ fontSize: "11px", color: "var(--neutralFg2)" }}>{node.label || shortId}</span>
                {i < sortedNodeIds.length - 1 && <span style={{ fontSize: "9px", color: "var(--neutralFg4)" }}>&rarr;</span>}
              </span>
            );
          })}
        </div>
      </div>

      {/* Output config */}
      <div style={{
        background: "var(--neutralBg2)", border: "1px solid var(--neutralStroke1)",
        borderRadius: "6px", padding: "8px",
      }}>
        <div style={{ fontSize: "10px", color: "var(--neutralFg4)", marginBottom: "4px" }}>Output</div>
        <div style={{ fontSize: "11px", color: "var(--neutralFg2)", display: "flex", flexDirection: "column", gap: "2px" }}>
          <span>Format: {outputSettings.format}</span>
          <span>Quality: {outputSettings.quality}%</span>
          {outputSettings.directory && <span>Dir: {outputSettings.directory}</span>}
          <span>Parallel: {outputSettings.parallel} threads</span>
        </div>
      </div>

      {/* Groups */}
      {groups.length > 0 && (
        <div style={{
          background: "var(--neutralBg2)", border: "1px solid var(--neutralStroke1)",
          borderRadius: "6px", padding: "8px",
        }}>
          <div style={{ fontSize: "10px", color: "var(--neutralFg4)", marginBottom: "4px" }}>Groups</div>
          <div style={{ fontSize: "11px", color: "var(--neutralFg2)", display: "flex", flexWrap: "wrap", gap: "6px" }}>
            {groups.map((g) => (
              <span key={g.name}>
                <span style={{ width: 6, height: 6, borderRadius: "50%", background: g.color, display: "inline-block", marginRight: 4 }} />
                {g.name} ({g.imageIndices.size})
              </span>
            ))}
          </div>
        </div>
      )}

      {nodes.length === 0 && (
        <div style={{ fontSize: "11px", color: "var(--neutralFg4)", textAlign: "center", padding: "16px 8px" }}>
          No pipeline defined.<br />Switch to Pipeline Editor to build one.
        </div>
      )}
    </div>
  );
}
