import type { BatchItem } from "../../stores/useBatchStore";
import "./BatchQueueRow.css";

interface BatchQueueRowProps {
  item: BatchItem;
  index: number;
}

export function BatchQueueRow({ item, index }: BatchQueueRowProps) {
  const { image, status, errorMessage } = item;

  const formatSize = (bytes: number) => {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(0)}MB`;
  };

  const statusText = status === "queued" ? "Queued"
    : status === "processing" ? "Processing..."
    : status === "done" ? "Done"
    : `Failed — ${errorMessage || "unknown error"}`;

  const isProcessing = status === "processing";

  return (
    <div className={`bqr ${isProcessing ? "bqr--processing" : ""}`}
      style={{ background: isProcessing ? "rgba(71,158,245,0.04)" : "var(--neutralBg2)", border: isProcessing ? "1px solid var(--brandFg1)" : "1px solid transparent" }}>
      <span className={`bqr-dot ${status}`} />
      <span className="bqr-name" title={image.filename}>{image.filename}</span>
      <span className="bqr-meta">{image.width}&times;{image.height} {image.format.toUpperCase()}</span>
      <span className="bqr-size">{formatSize(image.file_size_bytes)}</span>
      <span className={`bqr-status ${status === "failed" ? "bqr-status--err" : ""}`}>{statusText}</span>
    </div>
  );
}
