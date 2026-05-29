import type { AppSettings } from "../../types/settings";

interface TabProps {
  settings: AppSettings;
  update: <K extends keyof AppSettings>(k: K, v: AppSettings[K]) => void;
  onReset: () => void;
  showResetConfirm: boolean;
}

export function AdvancedTab({ settings, update, onReset, showResetConfirm }: TabProps) {
  return (
    <div>
      <div className="sd-row">
        <label className="sd-lbl">Tile Size</label>
        <input type="number" className="sd-inp" min={256} max={4096} value={settings.tileSize} onChange={(e) => update("tileSize", parseInt(e.target.value) || 1024)} />
      </div>
      <div className="sd-desc">Tile size for large image processing (256-4096 px)</div>
      <div className="sd-row">
        <label className="sd-lbl">Cache Directory</label>
        <input className="sd-inp" value={settings.cacheDirectory} onChange={(e) => update("cacheDirectory", e.target.value)} placeholder="%APPDATA%/Photopipeline/cache" />
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Max Cache Size</label>
        <input type="number" className="sd-inp" min={128} max={8192} value={settings.maxCacheSize} onChange={(e) => update("maxCacheSize", parseInt(e.target.value) || 1024)} />
      </div>
      <div className="sd-desc">Maximum cache size in MB (128-8192)</div>
      <div className="sd-row">
        <label className="sd-lbl">ExifTool Path</label>
        <input className="sd-inp" value={settings.exifToolPath} onChange={(e) => update("exifToolPath", e.target.value)} placeholder="exiftool" />
      </div>
      <div style={{ marginTop: "16px", paddingTop: "12px", borderTop: "1px solid var(--neutralStroke1)" }}>
        <button
          style={{ width: "100%", height: 32, background: "transparent", border: `1px solid ${showResetConfirm ? "var(--dangerFg)" : "var(--warningFg)"}`, borderRadius: "var(--radiusMedium)", color: showResetConfirm ? "var(--dangerFg)" : "var(--warningFg)", fontSize: "12px", cursor: "pointer" }}
          onClick={onReset}
          onMouseEnter={(e) => { e.currentTarget.style.background = showResetConfirm ? "rgba(209,52,71,0.08)" : "rgba(213,153,0,0.08)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
        >
          {showResetConfirm ? "⚠ Confirm: Reset ALL settings to factory defaults?" : "⚠ Reset All Settings to Defaults"}
        </button>
      </div>
    </div>
  );
}
