import { SliderWithInput } from "../common/SliderWithInput";
import { FilePathInput } from "../common/FilePathInput";
import { CoordinateInput } from "../common/CoordinateInput";
import { OverrideDot, type DotState } from "../common/OverrideDot";

export interface PanelWidget {
  type: string;
  paramId: string;
  label: string;
  description?: string;
  value: unknown;
  min?: number; max?: number; step?: number; precision?: number; unit?: string;
  placeholder?: string;
  options?: { value: string; label: string; description?: string; recommended?: boolean }[];
  filters?: [string, string][];
  kind?: string;
  labelOn?: string; labelOff?: string;
}

interface ParamRowProps {
  widget: PanelWidget;
  nodeId: string;
  dotState: DotState;
  canEdit: boolean;
  onChange: (paramId: string, value: unknown) => void;
  onActivate: (paramId: string) => void;
  onRestore: (paramId: string) => void;
  onExpressionEdit: (paramId: string) => void;
}

const fuiInputStyle: React.CSSProperties = {
  flex: 1, height: 30, background: "var(--neutralBg1)",
  border: "1px solid var(--neutralStroke1)", borderRadius: "var(--radiusMedium)",
  color: "var(--neutralFg1)", fontSize: "var(--fontSizeBody1)", padding: "0 8px",
  outline: "none",
};

export function ParamRow({ widget, dotState, canEdit, onChange, onActivate, onRestore, onExpressionEdit }: ParamRowProps) {
  const v = widget.value;
  const disabled = dotState === "inherited" || !canEdit;

  const renderControl = () => {
    switch (widget.type) {
      case "slider":
        return <SliderWithInput value={Number(v ?? 0)} min={widget.min ?? 0} max={widget.max ?? 100}
          step={widget.step} precision={widget.precision} unit={widget.unit}
          disabled={disabled} onChange={(val) => onChange(widget.paramId, val)} />;
      case "number_input":
        return <input type="number" style={fuiInputStyle} value={String(v ?? "")}
          min={widget.min} max={widget.max} step={widget.step} disabled={disabled}
          onChange={(e) => onChange(widget.paramId, parseFloat(e.target.value) || 0)} />;
      case "text_input":
        return <input type="text" style={fuiInputStyle} value={String(v ?? "")}
          placeholder={widget.placeholder} disabled={disabled}
          onChange={(e) => onChange(widget.paramId, e.target.value)} />;
      case "toggle":
        return (
          <label style={{ display: "inline-flex", alignItems: "center", gap: "8px", cursor: disabled ? "not-allowed" : "pointer", opacity: disabled ? 0.5 : 1 }}>
            <span style={{ fontSize: "11px", color: "var(--neutralFg2)", minWidth: 40 }}>
              {v ? (widget.labelOn ?? "ON") : (widget.labelOff ?? "OFF")}
            </span>
            <div className="sw-track" style={{ width: 36, height: 18, borderRadius: 9, background: v ? "var(--brandFg1)" : "var(--neutralStroke2)", position: "relative", transition: "background 0.2s" }}
              onClick={() => { if (!disabled) onChange(widget.paramId, !v); }}>
              <div className="sw-thumb" style={{ width: 14, height: 14, borderRadius: "50%", background: "#fff", position: "absolute", top: 2, left: v ? 20 : 2, transition: "left 0.2s", boxShadow: "0 1px 2px rgba(0,0,0,0.3)" }} />
            </div>
          </label>
        );
      case "dropdown":
        return <select style={fuiInputStyle} value={String(v ?? "")} disabled={disabled}
          onChange={(e) => onChange(widget.paramId, e.target.value)}>
          {widget.options?.map((o) => (
            <option key={o.value} value={o.value}>{o.label}{o.recommended ? " ★" : ""}</option>
          ))}
        </select>;
      case "file_picker":
        return <FilePathInput value={String(v ?? "")} kind={widget.kind as "file"|"directory"} filters={widget.filters}
          disabled={disabled} onChange={(val) => onChange(widget.paramId, val)} />;
      case "color_picker":
        return (
          <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
            <input type="color" value={String(v ?? "#000000")} disabled={disabled}
              style={{ width: 30, height: 30, border: "1px solid var(--neutralStroke1)", borderRadius: "var(--radiusMedium)", cursor: disabled ? "not-allowed" : "pointer", background: "transparent" }}
              onChange={(e) => onChange(widget.paramId, e.target.value)} />
            <input type="text" style={{ ...fuiInputStyle, width: 80 }} value={String(v ?? "")}
              disabled={disabled} onChange={(e) => onChange(widget.paramId, e.target.value)} />
          </div>
        );
      case "coordinate_input":
        return <CoordinateInput latitude={Number((v as any)?.lat ?? 0)} longitude={Number((v as any)?.lon ?? 0)}
          disabled={disabled} onChange={(coord) => onChange(widget.paramId, coord)} />;
      default:
        return <span style={{ fontSize: "11px", color: "var(--neutralFg4)", fontStyle: "italic", padding: "6px 0" }}>{widget.type}: {String(v)}</span>;
    }
  };

  return (
    <div style={{ marginBottom: "8px" }}>
      <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
        <label style={{
          width: 105, textAlign: "right", fontSize: "var(--fontSizeBody1)",
          color: "var(--neutralFg2)", flexShrink: 0, lineHeight: "30px",
        }}>{widget.label}</label>
        <div style={{ flex: 1, display: "flex", alignItems: "center", gap: "6px", minWidth: 0 }}>
          {renderControl()}
          <OverrideDot
            state={dotState} canEdit={canEdit}
            onActivate={() => onActivate(widget.paramId)}
            onRestore={() => onRestore(widget.paramId)}
            onExpressionEdit={() => onExpressionEdit(widget.paramId)}
          />
        </div>
      </div>
      {widget.description && (
        <div style={{ fontSize: "9px", color: "var(--neutralFg4)", marginLeft: 113, marginTop: 2, lineHeight: 1.4 }}>
          {widget.description}
        </div>
      )}
      {/* Source attribution for inherited values */}
      {dotState === "inherited" && canEdit && (
        <div style={{ fontSize: "9px", color: "var(--neutralFg4)", marginLeft: 113, marginTop: 1, fontStyle: "italic" }}>
          Inherited from Template
        </div>
      )}
    </div>
  );
}
