import { useState } from "react";
import { ParamRow, type PanelWidget } from "./ParamRow";
import { OverrideDot, type DotState } from "../common/OverrideDot";

export interface SectionData {
  id: string;
  label: string;
  collapsible: boolean;
  defaultCollapsed: boolean;
  widgets: PanelWidget[];
  // Override summary
  overrideBadge: "inherited" | "overrides" | "varying";
  overrideCount: number;
}

interface ParamSectionProps {
  section: SectionData;
  nodeId: string;
  getDotState: (paramId: string) => DotState;
  canEdit: boolean;
  onChange: (paramId: string, value: unknown) => void;
  onActivate: (paramId: string) => void;
  onRestore: (paramId: string) => void;
  onExpressionEdit: (paramId: string) => void;
}

export function ParamSection({ section, nodeId, getDotState, canEdit, onChange, onActivate, onRestore, onExpressionEdit }: ParamSectionProps) {
  const [collapsed, setCollapsed] = useState(section.defaultCollapsed && section.collapsible);

  if (section.collapsible) {
    return (
      <div className={`acc-item ${collapsed ? "collapsed" : ""}`} style={{ borderBottom: "1px solid var(--neutralStroke1)" }}>
        <div className="acc-header" style={{ display: "flex", alignItems: "center", padding: "8px 16px", cursor: "pointer", fontSize: "var(--fontSizeBody1)", color: "var(--neutralFg2)" }}
          onClick={() => setCollapsed(!collapsed)}>
          <span className="acc-caret" style={{ fontSize: 7, color: "var(--neutralFg3)", width: 14, transition: "transform 0.2s", transform: collapsed ? "rotate(-90deg)" : "none" }}>
            &#9660;
          </span>
          <span>{section.label}</span>
          {section.overrideBadge !== "inherited" && (
            <span style={{ marginLeft: 8, fontSize: 10, padding: "1px 6px", borderRadius: "var(--radiusSmall)", background: section.overrideBadge === "overrides" ? "rgba(213,153,0,0.08)" : "rgba(213,153,0,0.12)", color: "var(--warningFg)" }}>
              {section.overrideBadge === "overrides" ? `${section.overrideCount} overrides` : "values vary"}
            </span>
          )}
          {section.overrideBadge === "inherited" && (
            <span style={{ marginLeft: 8, fontSize: 10, color: "var(--neutralFg3)" }}>inherited</span>
          )}
        </div>
        {!collapsed && (
          <div className="acc-panel" style={{ padding: "0 16px 12px" }}>
            {section.widgets.map((w) => (
              <ParamRow key={w.paramId} widget={w} nodeId={nodeId} dotState={getDotState(w.paramId)}
                canEdit={canEdit} onChange={onChange} onActivate={onActivate} onRestore={onRestore} onExpressionEdit={onExpressionEdit} />
            ))}
          </div>
        )}
      </div>
    );
  }

  // Non-collapsible Card
  return (
    <div style={{ borderBottom: "1px solid var(--neutralStroke1)" }}>
      <div style={{ display: "flex", alignItems: "center", padding: "8px 16px", fontSize: "var(--fontSizeBody1)", fontWeight: 600, color: "var(--neutralFg1)" }}>
        <span>{section.label}</span>
        {section.overrideBadge !== "inherited" && (
          <span style={{ marginLeft: 8, fontSize: 10, padding: "1px 6px", borderRadius: "var(--radiusSmall)", background: "rgba(213,153,0,0.08)", color: "var(--warningFg)" }}>
            {section.overrideBadge === "overrides" ? `${section.overrideCount} overrides` : "values vary"}
          </span>
        )}
      </div>
      <div style={{ padding: "0 16px 12px" }}>
        {section.widgets.map((w) => (
          <ParamRow key={w.paramId} widget={w} nodeId={nodeId} dotState={getDotState(w.paramId)}
            canEdit={canEdit} onChange={onChange} onActivate={onActivate} onRestore={onRestore} onExpressionEdit={onExpressionEdit} />
        ))}
      </div>
    </div>
  );
}
