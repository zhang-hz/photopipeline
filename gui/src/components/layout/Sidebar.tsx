import { useFilmstripStore, type SortKey } from "../../stores/useFilmstripStore";
import { useBatchStore } from "../../stores/useBatchStore";
import { FilmstripList } from "../filmstrip/FilmstripList";
import { GroupTree } from "../filmstrip/GroupTree";

export function Sidebar() {
  const images = useFilmstripStore((s) => s.images);
  const selectedIndices = useFilmstripStore((s) => s.selectedIndices);
  const sortKey = useFilmstripStore((s) => s.sortKey);
  const thumbnailSize = useFilmstripStore((s) => s.thumbnailSize);
  const clearSelection = useFilmstripStore((s) => s.clearSelection);
  const setSortKey = useFilmstripStore((s) => s.setSortKey);
  const setThumbnailSize = useFilmstripStore((s) => s.setThumbnailSize);
  const createGroup = useFilmstripStore((s) => s.createGroup);
  const addToBatch = useBatchStore((s) => s.addToQueue);

  const handleToBatch = () => {
    const selected = images.filter((_, i) => selectedIndices.has(i));
    if (selected.length > 0) addToBatch(selected);
  };

  const handleAddToGroup = () => {
    createGroup(`分组 ${useFilmstripStore.getState().groups.length + 1}`);
  };

  return (
    <div className="sidebar">
      <div style={{ padding: "8px 16px", display: "flex", alignItems: "center", fontSize: "10px", fontWeight: 600, color: "var(--neutralFg3)", textTransform: "uppercase", letterSpacing: "0.6px", borderBottom: "1px solid var(--neutralStroke1)" }}>
        <span>候选文件</span>
        <span style={{ fontSize: "9px", color: "var(--neutralFg4)", marginLeft: "auto" }}>{images.length} 张图片</span>
      </div>

      <div style={{ padding: "8px 12px", display: "flex", gap: "4px", borderBottom: "1px solid var(--neutralStroke1)" }}>
        <button className="btn-primary-sm">&#128194; 导入</button>
        <button className="btn-subtle-sm" onClick={() => clearSelection()}>&#128465; 清除</button>
        <button className="btn-subtle-sm" onClick={handleToBatch} disabled={selectedIndices.size === 0}>&#128230; 批量</button>
      </div>

      <div style={{ padding: "4px 12px", display: "flex", gap: "8px", alignItems: "center", borderBottom: "1px solid var(--neutralStroke1)" }}>
        <span style={{ fontSize: "10px", color: "var(--neutralFg4)", flexShrink: 0 }}>排序:</span>
        <select value={sortKey} onChange={(e) => setSortKey(e.target.value as SortKey)} style={{ height: 22, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "10px", padding: "0 4px" }}>
          <option value="name">名称</option><option value="size">大小</option><option value="format">格式</option><option value="iso">ISO</option>
        </select>
        <span style={{ fontSize: "10px", color: "var(--neutralFg4)", flexShrink: 0, marginLeft: "auto" }}>尺寸:</span>
        {(["S", "M", "L"] as const).map((s) => (
          <button key={s} onClick={() => setThumbnailSize(s)} style={{ height: 22, padding: "0 6px", fontSize: "9px", fontFamily: "var(--fontFamily)", background: thumbnailSize === s ? "var(--neutralBg3)" : "transparent", border: `1px solid ${thumbnailSize === s ? "var(--neutralStroke2)" : "transparent"}`, borderRadius: "4px", color: thumbnailSize === s ? "var(--neutralFg1)" : "var(--neutralFg4)", cursor: "pointer", transition: "all var(--transitionFast)" }}>{s}</button>
        ))}
      </div>

      {selectedIndices.size > 1 && (
        <div style={{ padding: "4px 12px", background: "rgba(213,153,0,0.06)", color: "var(--warningFg)", fontSize: "10px", display: "flex", alignItems: "center", gap: "8px" }}>
          <span>&#128203; 已选中 {selectedIndices.size} 张</span>
          <span style={{ marginLeft: "auto", display: "flex", gap: "4px" }}>
            <button className="btn-subtle-sm" style={{ height: 20, fontSize: "9px", color: "var(--warningFg)" }} onClick={handleAddToGroup}>+分组</button>
            <button className="btn-subtle-sm" style={{ height: 20, fontSize: "9px", color: "var(--warningFg)" }} onClick={handleToBatch}>批量处理</button>
            <button className="btn-subtle-sm" style={{ height: 20, fontSize: "9px", color: "var(--warningFg)" }} onClick={clearSelection}>清除</button>
          </span>
        </div>
      )}

      {images.length > 0 ? (
        <FilmstripList />
      ) : (
        <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", color: "var(--neutralFg4)", gap: "8px" }}>
          <div style={{ fontSize: "40px", opacity: 0.25 }}>&#128193;</div>
          <div style={{ fontSize: "12px" }}>未加载图片</div>
          <div style={{ fontSize: "10px", color: "var(--neutralFg3)" }}>点击导入或拖放文件</div>
        </div>
      )}

      {images.length > 0 && <GroupTree />}
    </div>
  );
}
