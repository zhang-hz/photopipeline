import { useState } from "react";
import { useFilmstripStore } from "../../stores/useFilmstripStore";
import { useBatchStore } from "../../stores/useBatchStore";
import { ContextMenu, type MenuItem } from "../common/ContextMenu";
import type { ImageInfo } from "../../types/image";
import "./ImageCard.css";

interface ImageCardProps {
  image: ImageInfo;
  index: number;
  thumbnail?: string;
}

export function ImageCard({ image, index, thumbnail }: ImageCardProps) {
  const selectedIndices = useFilmstripStore((s) => s.selectedIndices);
  const groups = useFilmstripStore((s) => s.groups);
  const toggleSelect = useFilmstripStore((s) => s.toggleSelect);
  const selectAll = useFilmstripStore((s) => s.selectAll);
  const clearSelection = useFilmstripStore((s) => s.clearSelection);
  const invertSelection = useFilmstripStore((s) => s.invertSelection);
  const removeImages = useFilmstripStore((s) => s.removeImages);
  const addToBatch = useBatchStore((s) => s.addToQueue);

  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);

  const isSelected = selectedIndices.has(index);
  const isMultiSelect = selectedIndices.size > 1 && isSelected;
  const isSingleSelect = selectedIndices.size === 1 && isSelected;
  const imageGroups = groups.filter((g) => g.imageIndices.has(index));
  const stateClass = isMultiSelect ? "ic--multi" : isSingleSelect ? "ic--sel" : "";

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)}KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)}MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)}GB`;
  };

  const ctxItems: MenuItem[] = [
    { type: "item", label: "Open in Explorer", onClick: () => {} },
    { type: "item", label: "Copy Path", onClick: () => {} },
    { type: "separator" },
    { type: "item", label: "Select All", shortcut: "Ctrl+A", onClick: selectAll },
    { type: "item", label: "Clear Selection", shortcut: "Esc", onClick: clearSelection },
    { type: "item", label: "Invert Selection", onClick: invertSelection },
    { type: "separator" },
    {
      type: "item", label: "Add to Group", children: [
        ...groups.map((g) => ({ type: "item" as const, label: g.name, onClick: () => {} })),
        { type: "item" as const, label: "+ New Group", onClick: () => {} },
      ],
    },
    {
      type: "item", label: "Send to Batch",
      onClick: () => {
        const { images } = useFilmstripStore.getState();
        const selected = images.filter((_, i) => selectedIndices.has(i));
        if (selected.length > 0) addToBatch(selected);
      },
    },
    { type: "separator" },
    { type: "item", label: "Remove", shortcut: "Del", danger: true, onClick: () => removeImages([index]) },
  ];

  return (
    <>
      <div
        className={`ic ${stateClass}`}
        onClick={(e) => toggleSelect(index, e.ctrlKey || e.metaKey, e.shiftKey)}
        onContextMenu={(e) => { e.preventDefault(); setCtxMenu({ x: e.clientX, y: e.clientY }); }}
        draggable
      >
        {isMultiSelect && <div className="ic-chk">&#10003;</div>}
        <div className="ic-th">
          {thumbnail ? (
            <img src={thumbnail} alt="" className="ic-th-img" />
          ) : (
            <span className="ic-th-icon">&#128247;</span>
          )}
        </div>
        <div className="ic-info">
          <div className="ic-name" title={image.filename}>{image.filename}</div>
          <div className="ic-meta">
            {image.width}&times;{image.height}
            <span className="ic-meta-sep">&middot;</span>
            {image.format.toUpperCase()}
            {image.metadata?.iso != null && (
              <><span className="ic-meta-sep">&middot;</span>ISO {image.metadata.iso}</>
            )}
          </div>
          {imageGroups.map((g) => (
            <span key={g.name} className="ic-tag" style={{ background: `${g.color}15`, color: g.color }}>{g.name}</span>
          ))}
        </div>
        <div className="ic-size">{formatSize(image.file_size_bytes)}</div>
      </div>
      <ContextMenu
        isOpen={ctxMenu !== null}
        x={ctxMenu?.x ?? 0}
        y={ctxMenu?.y ?? 0}
        items={ctxItems}
        onClose={() => setCtxMenu(null)}
      />
    </>
  );
}
