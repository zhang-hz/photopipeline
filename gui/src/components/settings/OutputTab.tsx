import type { AppSettings } from "../../types/settings";

interface TabProps { settings: AppSettings; update: <K extends keyof AppSettings>(k: K, v: AppSettings[K]) => void; }

export function OutputTab({ settings, update }: TabProps) {
  return (
    <div>
      <div className="sd-row">
        <label className="sd-lbl">Default Format</label>
        <select className="sd-inp" value={settings.defaultFormat} onChange={(e) => update("defaultFormat", e.target.value)}>
          <option>HEIF</option><option>JXL</option><option>AVIF</option><option>TIFF</option><option>PNG</option>
        </select>
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Default Directory</label>
        <input className="sd-inp" value={settings.defaultDirectory} onChange={(e) => update("defaultDirectory", e.target.value)} placeholder="Leave empty for last used" />
      </div>
      <div className="sd-row">
        <label className="sd-lbl">JPEG Quality</label>
        <input type="range" min={0} max={100} value={settings.jpegQuality} style={{ flex: 1, height: 4 }} onChange={(e) => update("jpegQuality", parseInt(e.target.value))} />
        <span style={{ width: 36, fontSize: "11px", color: "var(--neutralFg2)", textAlign: "right" }}>{settings.jpegQuality}%</span>
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Embed Metadata</label>
        <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
          <div style={{ width: 36, height: 18, borderRadius: 9, background: settings.embedMetadata ? "var(--brandFg1)" : "var(--neutralStroke2)", position: "relative", transition: "background 0.2s" }}
            onClick={() => update("embedMetadata", !settings.embedMetadata)}>
            <div style={{ width: 14, height: 14, borderRadius: "50%", background: "#fff", position: "absolute", top: 2, left: settings.embedMetadata ? 20 : 2, transition: "left 0.2s", boxShadow: "0 1px 2px rgba(0,0,0,0.3)" }} />
          </div>
          <span style={{ fontSize: "11px", color: "var(--neutralFg2)" }}>{settings.embedMetadata ? "On" : "Off"}</span>
        </label>
      </div>
      <div className="sd-desc">Embed EXIF/XMP/IPTC metadata in output files</div>
      <div className="sd-row">
        <label className="sd-lbl">Thumbnail Size</label>
        <input type="number" className="sd-inp" min={64} max={512} value={settings.thumbnailSize} onChange={(e) => update("thumbnailSize", parseInt(e.target.value) || 120)} />
      </div>
      <div className="sd-desc">Default thumbnail max side length (64-512 px)</div>
    </div>
  );
}
