import "./SliderWithInput.css";

interface SliderWithInputProps {
  value: number;
  min: number;
  max: number;
  step?: number;
  precision?: number;
  unit?: string;
  disabled?: boolean;
  onChange: (value: number) => void;
}

export function SliderWithInput({ value, min, max, step = 1, precision = 0, unit, disabled, onChange }: SliderWithInputProps) {
  const pct = ((value - min) / (max - min)) * 100;
  const displayValue = precision > 0 ? value.toFixed(precision) : String(Math.round(value));

  return (
    <div className={`siw ${disabled ? "siw--disabled" : ""}`}>
      <input
        type="range"
        className="siw-slider"
        min={min} max={max} step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        disabled={disabled}
        style={{ background: `linear-gradient(to right, var(--brandFg1) ${pct}%, var(--neutralStroke2) ${pct}%)` }}
      />
      <span className="siw-value">{displayValue}</span>
      {unit && <span className="siw-unit">{unit}</span>}
    </div>
  );
}
