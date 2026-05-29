import { useState } from "react";
import { useFilmstripStore } from "../../stores/useFilmstripStore";

export function GroupTree() {
  const groups = useFilmstripStore((s) => s.groups);
  const createGroup = useFilmstripStore((s) => s.createGroup);
  const deleteGroup = useFilmstripStore((s) => s.deleteGroup);
  const [hoveredGroup, setHoveredGroup] = useState<string | null>(null);

  return (
    <div style={{ padding: "0", borderTop: "1px solid var(--neutralStroke1)" }}>
      <div style={{
        padding: "6px 12px", fontSize: "var(--fontSizeCaption1)", fontWeight: 600,
        color: "var(--neutralFg4)", textTransform: "uppercase",
      }}>
        Groups
      </div>

      {groups.map((g) => (
        <div
          key={g.name}
          className="grp-item"
          style={{
            padding: "5px 12px", fontSize: "var(--fontSizeBody1)", color: "var(--neutralFg2)",
            cursor: "pointer", display: "flex", alignItems: "center", gap: "6px",
            background: hoveredGroup === g.name ? "var(--neutralBg3)" : "transparent",
          }}
          onMouseEnter={() => setHoveredGroup(g.name)}
          onMouseLeave={() => setHoveredGroup(null)}
        >
          <span style={{ width: 6, height: 6, borderRadius: "50%", background: g.color, flexShrink: 0 }} />
          <span style={{ flex: 1 }}>{g.name}</span>
          <span style={{ fontSize: 9, background: "var(--neutralBg3)", padding: "1px 5px", borderRadius: "var(--radiusSmall)" }}>
            {g.imageIndices.size}
          </span>
          {hoveredGroup === g.name && (
            <span style={{ fontSize: "11px", display: "flex", gap: "4px" }}>
              <span title="Edit group">&#9998;</span>
              <span title="Delete group" onClick={() => deleteGroup(g.name)} style={{ cursor: "pointer" }}>&#128465;</span>
            </span>
          )}
        </div>
      ))}

      <div style={{ padding: "4px 12px", display: "flex", flexDirection: "column", gap: "4px" }}>
        <button
          style={{
            width: "100%", padding: "4px 8px", background: "transparent",
            border: "1px dashed var(--neutralStroke1)", borderRadius: "var(--radiusMedium)",
            color: "var(--neutralFg2)", fontSize: "11px", cursor: "pointer",
          }}
          onClick={() => createGroup(`Group ${groups.length + 1}`)}
        >
          + Create Group&hellip;
        </button>
      </div>
    </div>
  );
}
