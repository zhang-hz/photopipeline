import type { AppSettings } from "../../types/settings";

interface TabProps { settings: AppSettings; update: <K extends keyof AppSettings>(k: K, v: AppSettings[K]) => void; }

export function BackendTab({ settings, update }: TabProps) {
  return (
    <div>
      <div className="sd-row">
        <label className="sd-lbl">Server Path</label>
        <input className="sd-inp" value={settings.serverPath} onChange={(e) => update("serverPath", e.target.value)} />
      </div>
      <div className="sd-desc">Path to the photopipeline binary</div>
      <div className="sd-row">
        <label className="sd-lbl">Port</label>
        <input type="number" className="sd-inp" min={1024} max={65535} value={settings.port} onChange={(e) => update("port", parseInt(e.target.value) || 50051)} />
      </div>
      <div className="sd-desc">gRPC server port (1024-65535, default 50051)</div>
      <div className="sd-row">
        <label className="sd-lbl">Auto-start</label>
        <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
          <div style={{ width: 36, height: 18, borderRadius: 9, background: settings.autoStart ? "var(--brandFg1)" : "var(--neutralStroke2)", position: "relative", transition: "background 0.2s" }}
            onClick={() => update("autoStart", !settings.autoStart)}>
            <div style={{ width: 14, height: 14, borderRadius: "50%", background: "#fff", position: "absolute", top: 2, left: settings.autoStart ? 20 : 2, transition: "left 0.2s", boxShadow: "0 1px 2px rgba(0,0,0,0.3)" }} />
          </div>
          <span style={{ fontSize: "11px", color: "var(--neutralFg2)" }}>{settings.autoStart ? "On" : "Off"}</span>
        </label>
      </div>
      <div className="sd-desc">Automatically start backend on app launch</div>
      <div className="sd-row">
        <label className="sd-lbl">GPU Backend</label>
        <select className="sd-inp" value={settings.gpuBackend} onChange={(e) => update("gpuBackend", e.target.value)}>
          <option>Auto</option><option>CUDA</option><option>CPU</option><option>CoreML</option><option>OpenVINO</option>
        </select>
      </div>
      <div className="sd-row">
        <label className="sd-lbl">Log Level</label>
        <select className="sd-inp" value={settings.logLevel} onChange={(e) => update("logLevel", e.target.value)}>
          <option>Info</option><option>Debug</option><option>Warn</option><option>Error</option>
        </select>
      </div>
    </div>
  );
}
