import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useBatchStore } from "../../stores/useBatchStore";
import { BatchQueueRow } from "./BatchQueueRow";

export function BatchQueueList() {
  const queue = useBatchStore((s) => s.queue);
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: queue.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40,
    overscan: 10,
  });

  if (queue.length === 0) {
    return (
      <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--neutralFg4)", fontSize: "12px" }}>
        No items in queue. Select images and click "To Batch" or "Send to Batch".
      </div>
    );
  }

  return (
    <div ref={parentRef} style={{ flex: 1, overflow: "auto" }}>
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {virtualizer.getVirtualItems().map((vItem) => (
          <div key={vItem.key} style={{ position: "absolute", top: 0, left: 0, width: "100%", transform: `translateY(${vItem.start}px)` }}>
            <BatchQueueRow item={queue[vItem.index]} index={vItem.index} />
          </div>
        ))}
      </div>
    </div>
  );
}
