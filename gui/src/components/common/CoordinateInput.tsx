interface CoordinateInputProps {
  latitude: number;
  longitude: number;
  altitude?: number;
  altRequired?: boolean;
  disabled?: boolean;
  onChange: (coord: { lat: number; lon: number; alt?: number }) => void;
}

export function CoordinateInput({ latitude, longitude, altitude, altRequired, disabled, onChange }: CoordinateInputProps) {
  return (
    <div style={{ display: "flex", gap: "6px", flex: 1 }}>
      <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
        <span style={{ fontSize: "10px", color: "var(--neutralFg4)", fontFamily: "var(--fontFamilyMono)" }}>lat</span>
        <input type="number" value={latitude} disabled={disabled} min={-90} max={90} step={0.000001}
          style={{ width: 90, height: 30, background: disabled ? "var(--neutralBg2)" : "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "var(--radiusMedium)", color: disabled ? "var(--neutralFg4)" : "var(--neutralFg1)", fontSize: "11px", padding: "0 6px", fontFamily: "var(--fontFamilyMono)", textAlign: "right" }}
          onChange={(e) => onChange({ lat: parseFloat(e.target.value) || 0, lon: longitude, alt: altitude })} />
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
        <span style={{ fontSize: "10px", color: "var(--neutralFg4)", fontFamily: "var(--fontFamilyMono)" }}>lon</span>
        <input type="number" value={longitude} disabled={disabled} min={-180} max={180} step={0.000001}
          style={{ width: 90, height: 30, background: disabled ? "var(--neutralBg2)" : "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "var(--radiusMedium)", color: disabled ? "var(--neutralFg4)" : "var(--neutralFg1)", fontSize: "11px", padding: "0 6px", fontFamily: "var(--fontFamilyMono)", textAlign: "right" }}
          onChange={(e) => onChange({ lat: latitude, lon: parseFloat(e.target.value) || 0, alt: altitude })} />
      </div>
      {(altitude !== undefined || altRequired) && (
        <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
          <span style={{ fontSize: "10px", color: "var(--neutralFg4)", fontFamily: "var(--fontFamilyMono)" }}>alt</span>
          <input type="number" value={altitude ?? 0} disabled={disabled} min={-500} max={9000} step={0.1}
            style={{ width: 70, height: 30, background: disabled ? "var(--neutralBg2)" : "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)", borderRadius: "var(--radiusMedium)", color: disabled ? "var(--neutralFg4)" : "var(--neutralFg1)", fontSize: "11px", padding: "0 6px", fontFamily: "var(--fontFamilyMono)", textAlign: "right" }}
            onChange={(e) => onChange({ lat: latitude, lon: longitude, alt: parseFloat(e.target.value) || 0 })} />
          <span style={{ fontSize: "10px", color: "var(--neutralFg4)" }}>m</span>
        </div>
      )}
    </div>
  );
}
