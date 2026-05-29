import { usePluginStore, useFilteredPlugins } from "../../stores/usePluginStore";
import { PluginCard } from "./PluginCard";

interface PluginBrowserProps {
  onAddNode?: (pluginId: string) => void;
}

export function PluginBrowser({ onAddNode }: PluginBrowserProps) {
  const categories = usePluginStore((s) => s.categories);
  const searchQuery = usePluginStore((s) => s.searchQuery);
  const categoryFilter = usePluginStore((s) => s.categoryFilter);
  const setSearch = usePluginStore((s) => s.setSearch);
  const setCategoryFilter = usePluginStore((s) => s.setCategoryFilter);
  const filteredPlugins = useFilteredPlugins();

  return (
    <div style={{ borderTop: "1px solid var(--neutralStroke1)", borderBottom: "1px solid var(--neutralStroke1)", background: "var(--neutralBg1)", padding: "8px 12px" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "6px" }}>
        <span style={{ fontSize: "10px", fontWeight: 600, color: "var(--neutralFg3)", textTransform: "uppercase" }}>Plugins</span>
        <input type="text" placeholder="Search plugins..." value={searchQuery} onChange={(e) => setSearch(e.target.value)}
          style={{ width: 160, height: 28, background: "var(--neutralBg2)", border: "1px solid var(--neutralStroke1)", borderRadius: "var(--radiusMedium)", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 8px" }} />
        <select value={categoryFilter} onChange={(e) => setCategoryFilter(e.target.value)}
          style={{ width: 96, height: 28, background: "var(--neutralBg2)", border: "1px solid var(--neutralStroke1)", borderRadius: "var(--radiusMedium)", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 4px" }}>
          <option value="All">All</option>
          {categories.map((c) => <option key={c} value={c}>{c}</option>)}
        </select>
      </div>
      <div style={{ display: "flex", gap: "4px", overflowX: "auto", paddingBottom: "2px" }}>
        {filteredPlugins.map((p) => (
          <PluginCard key={p.id} plugin={p} onDoubleClick={onAddNode} />
        ))}
      </div>
    </div>
  );
}
