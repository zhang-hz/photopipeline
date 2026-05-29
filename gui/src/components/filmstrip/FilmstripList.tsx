import { useRef, useMemo } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useFilmstripStore } from "../../stores/useFilmstripStore";
import { ImageCard } from "./ImageCard";

export function FilmstripList() {
  const images = useFilmstripStore((s) => s.images);
  const sortKey = useFilmstripStore((s) => s.sortKey);
  const thumbnailSize = useFilmstripStore((s) => s.thumbnailSize);
  const parentRef = useRef<HTMLDivElement>(null);

  const sorted = useMemo(() => {
    const arr = images.map((img, i) => ({ img, i }));
    switch (sortKey) {
      case "name": arr.sort((a, b) => a.img.filename.localeCompare(b.img.filename)); break;
      case "size": arr.sort((a, b) => b.img.file_size_bytes - a.img.file_size_bytes); break;
      case "format": arr.sort((a, b) => a.img.format.localeCompare(b.img.format)); break;
      case "iso": arr.sort((a, b) => (b.img.metadata?.iso ?? 0) - (a.img.metadata?.iso ?? 0)); break;
    }
    return arr;
  }, [images, sortKey]);

  const rowHeight = thumbnailSize === "S" ? 60 : thumbnailSize === "L" ? 85 : 70;

  const virtualizer = useVirtualizer({
    count: sorted.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => rowHeight,
    overscan: 5,
  });

  return (
    <div ref={parentRef} style={{ flex: 1, overflow: "auto", padding: "4px" }}>
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {virtualizer.getVirtualItems().map((vItem) => (
          <div
            key={sorted[vItem.index].img.id}
            style={{ position: "absolute", top: 0, left: 0, width: "100%", transform: `translateY(${vItem.start}px)` }}
          >
            <ImageCard image={sorted[vItem.index].img} index={sorted[vItem.index].i} />
          </div>
        ))}
      </div>
    </div>
  );
}
