interface FilePathInputProps {
  value: string;
  placeholder?: string;
  kind?: "file" | "directory";
  filters?: [string, string][];
  disabled?: boolean;
  onChange: (value: string) => void;
}

export function FilePathInput({ value, placeholder, disabled, onChange }: FilePathInputProps) {
  return (
    <div style={{ display: "flex", gap: "4px", flex: 1 }}>
      <input
        type="text"
        className="fui-input"
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        style={{
          flex: 1, height: 30, background: "var(--neutralBg1)", border: "1px solid var(--neutralStroke1)",
          borderRadius: "var(--radiusMedium)", color: "var(--neutralFg1)", fontSize: "12px", padding: "0 8px",
        }}
      />
      <button
        className="btn-subtle-sm"
        disabled={disabled}
        style={{ height: 30, width: 32, flexShrink: 0 }}
        title="Browse..."
        onClick={() => {/* TODO: Tauri dialog.open */}}
      >
        &#128194;
      </button>
    </div>
  );
}
