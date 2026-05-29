import { useState } from "react";
import { useBatchStore } from "../../stores/useBatchStore";

export function BatchRightPanel() {
  const outputSettings = useBatchStore((s) => s.outputSettings);
  const setOutputSetting = useBatchStore((s) => s.setOutputSetting);
  const queue = useBatchStore((s) => s.queue);
  const perImageOverrides = useBatchStore((s) => s.perImageOverrides);
  const setPerImageOverride = useBatchStore((s) => s.setPerImageOverride);
  const [selectedImageId, setSelectedImageId] = useState<string>("");

  const selectedOverrides = selectedImageId ? perImageOverrides.get(selectedImageId) ?? {} : {};

  const pendingImages = queue.filter((q) => q.status === "queued");

  return (
    <div className="batch-right">
      <div style={{ fontSize: "10px", fontWeight: 600, color: "var(--neutralFg3)", textTransform: "uppercase", letterSpacing: "0.6px" }}>
        Output Settings
      </div>

      {/* Directory */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <label style={{ width: 72, textAlign: "right", fontSize: "10px", color: "var(--neutralFg2)", flexShrink: 0 }}>Directory</label>
        <input style={{ flex: 1, height: 26, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 8px" }}
          value={outputSettings.directory} onChange={(e) => setOutputSetting("directory", e.target.value)} placeholder="C:\output" />
        <button className="btn-subtle-sm" style={{ height: 26, width: 28 }}>&hellip;</button>
      </div>

      {/* Template */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <label style={{ width: 72, textAlign: "right", fontSize: "10px", color: "var(--neutralFg2)", flexShrink: 0 }}>Template</label>
        <input style={{ flex: 1, height: 26, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 8px" }}
          value={outputSettings.template} onChange={(e) => setOutputSetting("template", e.target.value)} />
      </div>

      {/* Format */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <label style={{ width: 72, textAlign: "right", fontSize: "10px", color: "var(--neutralFg2)", flexShrink: 0 }}>Format</label>
        <select style={{ flex: 1, height: 26, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 4px" }}
          value={outputSettings.format} onChange={(e) => setOutputSetting("format", e.target.value)}>
          <option>HEIF</option><option>JXL</option><option>AVIF</option><option>TIFF</option><option>PNG</option>
        </select>
      </div>

      {/* Quality */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <label style={{ width: 72, textAlign: "right", fontSize: "10px", color: "var(--neutralFg2)", flexShrink: 0 }}>Quality</label>
        <input type="range" min={0} max={100} value={outputSettings.quality}
          style={{ flex: 1, height: 4 }}
          onChange={(e) => setOutputSetting("quality", parseInt(e.target.value))} />
        <span style={{ width: 36, fontSize: "10px", color: "var(--neutralFg2)", textAlign: "right" }}>{outputSettings.quality}%</span>
      </div>

      {/* Parallel */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <label style={{ width: 72, textAlign: "right", fontSize: "10px", color: "var(--neutralFg2)", flexShrink: 0 }}>Parallel</label>
        <input type="number" min={1} max={32} value={outputSettings.parallel}
          style={{ width: 56, height: 26, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 4px", textAlign: "center" }}
          onChange={(e) => setOutputSetting("parallel", parseInt(e.target.value) || 1)} />
        <span style={{ fontSize: "10px", color: "var(--neutralFg4)" }}>threads</span>
      </div>

      {/* Conflict */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <label style={{ width: 72, textAlign: "right", fontSize: "10px", color: "var(--neutralFg2)", flexShrink: 0 }}>Conflict</label>
        <select style={{ flex: 1, height: 26, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 4px" }}
          value={outputSettings.conflict} onChange={(e) => setOutputSetting("conflict", e.target.value as any)}>
          <option value="skip">Skip existing</option>
          <option value="overwrite">Overwrite</option>
          <option value="rename">Rename new</option>
        </select>
      </div>

      {/* Per-Image Override */}
      <div style={{ borderTop: "1px solid var(--neutralStroke1)", paddingTop: "12px", marginTop: "4px" }}>
        <div style={{ fontSize: "10px", fontWeight: 600, color: "var(--neutralFg3)", textTransform: "uppercase", letterSpacing: "0.6px", marginBottom: "8px" }}>
          Per-Image Override
        </div>
        <select
          style={{ width: "100%", height: 28, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 8px", marginBottom: "8px" }}
          value={selectedImageId} onChange={(e) => setSelectedImageId(e.target.value)}>
          <option value="">Select queued image...</option>
          {pendingImages.map((q) => (
            <option key={q.image.id} value={q.image.id}>{q.image.filename}</option>
          ))}
        </select>

        {selectedImageId && (
          <div style={{ background: "rgba(213,153,0,0.04)", border: "1px solid rgba(213,153,0,0.12)", borderRadius: "6px", padding: "8px" }}>
            <div style={{ fontSize: "11px", fontWeight: 600, color: "var(--warningFg)", marginBottom: "6px" }}>
              Overrides for {queue.find((q) => q.image.id === selectedImageId)?.image.filename}
            </div>
            {Object.keys(selectedOverrides).length > 0 ? (
              Object.entries(selectedOverrides).map(([key, val]) => (
                <div key={key} style={{ display: "flex", alignItems: "center", gap: "6px", marginBottom: "4px" }}>
                  <span style={{ fontSize: "10px", color: "var(--neutralFg4)", flex: 1 }}>{key}</span>
                  <input
                    style={{ width: 80, height: 24, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "4px", color: "var(--neutralFg1)", fontSize: "11px", padding: "0 6px", textAlign: "right" }}
                    value={String(val)}
                    onChange={(e) => setPerImageOverride(selectedImageId, "denoise_1", key, e.target.value)}
                  />
                  <span style={{ width: 8, height: 8, borderRadius: "50%", background: "var(--warningFg)", flexShrink: 0 }} />
                </div>
              ))
            ) : (
              <div style={{ fontSize: "10px", color: "var(--neutralFg4)", textAlign: "center", padding: "4px" }}>
                No overrides set
              </div>
            )}
            <button className="btn-subtle-sm" style={{ width: "100%", height: 26, border: "1px dashed var(--neutralStroke1)", marginTop: "4px", fontSize: "10px" }}
              onClick={() => setPerImageOverride(selectedImageId, "denoise_1", "strength", 0.5)}>
              + Add parameter override
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
