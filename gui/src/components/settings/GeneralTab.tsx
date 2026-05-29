import type { AppSettings } from "../../types/settings";

interface TabProps { settings: AppSettings; update: <K extends keyof AppSettings>(k: K, v: AppSettings[K]) => void; }

export function GeneralTab({ settings, update }: TabProps) {
  return (
    <div>
      <div className="sd-row">
        <label className="sd-lbl">Theme</label>
        <select className="sd-inp" value={settings.theme} onChange={(e) => update("theme", e.target.value)}>
          <option value="dark">Dark</option><option value="light">Light</option><option value="system">System</option>
        </select>
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Language</label>
        <select className="sd-inp" value={settings.language} onChange={(e) => update("language", e.target.value)}>
          <option value="English">English</option><option value="中文">中文</option><option value="日本語">日本語</option>
        </select>
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Max Recent Files</label>
        <input type="number" className="sd-inp" min={5} max={50} value={settings.maxRecentFiles} onChange={(e) => update("maxRecentFiles", parseInt(e.target.value) || 10)} />
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Check Updates</label>
        <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
          <div style={{ width: 36, height: 18, borderRadius: 9, background: settings.checkUpdates ? "var(--brandFg1)" : "var(--neutralStroke2)", position: "relative", transition: "background 0.2s" }}
            onClick={() => update("checkUpdates", !settings.checkUpdates)}>
            <div style={{ width: 14, height: 14, borderRadius: "50%", background: "#fff", position: "absolute", top: 2, left: settings.checkUpdates ? 20 : 2, transition: "left 0.2s", boxShadow: "0 1px 2px rgba(0,0,0,0.3)" }} />
          </div>
          <span style={{ fontSize: "11px", color: "var(--neutralFg2)" }}>{settings.checkUpdates ? "On" : "Off"}</span>
        </label>
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Telemetry</label>
        <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
          <div style={{ width: 36, height: 18, borderRadius: 9, background: settings.telemetry ? "var(--brandFg1)" : "var(--neutralStroke2)", position: "relative", transition: "background 0.2s" }}
            onClick={() => update("telemetry", !settings.telemetry)}>
            <div style={{ width: 14, height: 14, borderRadius: "50%", background: "#fff", position: "absolute", top: 2, left: settings.telemetry ? 20 : 2, transition: "left 0.2s", boxShadow: "0 1px 2px rgba(0,0,0,0.3)" }} />
          </div>
          <span style={{ fontSize: "11px", color: "var(--neutralFg2)" }}>{settings.telemetry ? "On" : "Off"}</span>
        </label>
      </div>
    </div>
  );
}
