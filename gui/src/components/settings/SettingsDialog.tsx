import { useState } from "react";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { useAppStore } from "../../stores/useAppStore";
import { GeneralTab } from "./GeneralTab";
import { BackendTab } from "./BackendTab";
import { OutputTab } from "./OutputTab";
import { AdvancedTab } from "./AdvancedTab";
import "./SettingsDialog.css";

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const TABS = ["General", "Backend", "Output", "Advanced"];

export function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const [activeTab, setActiveTab] = useState("General");
  const settings = useSettingsStore((s) => s.settings);
  const isDirty = useSettingsStore((s) => s.isDirty);
  const save = useSettingsStore((s) => s.save);
  const reset = useSettingsStore((s) => s.reset);
  const cancel = useSettingsStore((s) => s.cancel);
  const update = useSettingsStore((s) => s.update);
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  if (!isOpen) return null;

  const handleSave = () => {
    save();
    // Apply theme immediately
    useAppStore.getState().setTheme(settings.theme as "dark" | "light" | "system");
    onClose();
  };

  const handleCancel = () => {
    cancel();
    onClose();
  };

  const handleReset = () => {
    if (!showResetConfirm) { setShowResetConfirm(true); return; }
    reset();
    setShowResetConfirm(false);
  };

  const renderTab = () => {
    switch (activeTab) {
      case "General": return <GeneralTab settings={settings} update={update} />;
      case "Backend": return <BackendTab settings={settings} update={update} />;
      case "Output": return <OutputTab settings={settings} update={update} />;
      case "Advanced": return <AdvancedTab settings={settings} update={update} onReset={handleReset} showResetConfirm={showResetConfirm} />;
      default: return null;
    }
  };

  return (
    <div className="sd-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="sd-dialog">
        {/* Header */}
        <div className="sd-header">
          <span style={{ fontSize: "13px", fontWeight: 600 }}>Settings</span>
          <button className="tb-btn" style={{ width: 28, height: 28, fontSize: 12 }} onClick={onClose}>&#10005;</button>
        </div>

        {/* Tabs */}
        <div className="sd-tabs">
          {TABS.map((t) => (
            <button key={t} className={`sd-tab ${activeTab === t ? "active" : ""}`} onClick={() => setActiveTab(t)}>{t}</button>
          ))}
        </div>

        {/* Content */}
        <div className="sd-content">
          {renderTab()}
        </div>

        {/* Footer */}
        <div className="sd-footer">
          {isDirty && <span style={{ fontSize: "10px", color: "var(--warningFg)", marginRight: "auto" }}>Unsaved changes</span>}
          <button className="btn-subtle-sm" onClick={handleCancel}>Cancel</button>
          {activeTab === "Advanced" && (
            <button className="btn-subtle-sm" style={{ color: showResetConfirm ? "var(--dangerFg)" : "var(--warningFg)", borderColor: showResetConfirm ? "var(--dangerFg)" : "var(--warningFg)" }}
              onClick={handleReset}>{showResetConfirm ? "Confirm Reset All?" : "Reset All"}</button>
          )}
          {activeTab !== "Advanced" && <button className="btn-subtle-sm" onClick={handleReset}>Reset</button>}
          <button className="btn-primary-sm" onClick={handleSave} disabled={!isDirty}>Save</button>
        </div>
      </div>
    </div>
  );
}
