import { useOverrideStore, type OverrideScope } from "../../stores/useOverrideStore";
import { useFilmstripStore } from "../../stores/useFilmstripStore";

export function ContextBar() {
  const scope = useOverrideStore((s) => s.scope);
  const setScope = useOverrideStore((s) => s.setScope);
  const images = useFilmstripStore((s) => s.images);
  const selectedIndices = useFilmstripStore((s) => s.selectedIndices);
  const groups = useFilmstripStore((s) => s.groups);

  const scopes: { id: OverrideScope; label: string; groupName?: string }[] = [
    { id: "all", label: "All" },
    { id: "template", label: "Template" },
    ...groups.map((g) => ({ id: "group" as OverrideScope, label: g.name, groupName: g.name })),
  ];

  // Only show image scope when single image selected
  if (selectedIndices.size === 1) {
    const idx = [...selectedIndices][0];
    if (images[idx]) {
      scopes.push({ id: "image" as OverrideScope, label: images[idx].filename });
    }
  } else if (selectedIndices.size > 1) {
    scopes.push({ id: "image" as OverrideScope, label: `${selectedIndices.size} images` });
  }

  return (
    <div className="ctx-bar" style={{
      display: "flex", gap: 0, padding: "0 8px", borderBottom: "1px solid var(--neutralStroke1)",
      overflowX: "auto", flexShrink: 0,
    }}>
      {scopes.map((s) => {
        const isActive = scope === s.id
          && (s.id === "all" || s.id === "template"
            || (s.id === "group" && useOverrideStore.getState().activeGroupName === s.groupName)
            || s.id === "image");
        return (
          <button
            key={s.label}
            className={`ctx-tab ${isActive ? "ctx-tab--active" : ""}`}
            style={{
              padding: "8px 12px", fontSize: "var(--fontSizeBody1)", border: "none",
              background: "transparent", cursor: "pointer", whiteSpace: "nowrap",
              color: isActive ? "var(--brandFg1)" : "var(--neutralFg2)",
              borderBottom: isActive ? "2px solid var(--brandFg1)" : "2px solid transparent",
              transition: "all var(--transitionFast)",
            }}
            onClick={() => setScope(s.id, s.groupName)}
          >
            {s.label}
          </button>
        );
      })}
    </div>
  );
}
