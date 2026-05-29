import { useState, useRef, useCallback, useEffect } from "react";

interface ExpressionEditorProps {
  expression: string;
  variables?: { name: string; description: string; example?: string }[];
  previews?: { label: string; result: number }[];
  onChange: (expr: string) => void;
  onClose: () => void;
}

export function ExpressionEditor({ expression, variables = [], previews = [], onChange, onClose }: ExpressionEditorProps) {
  const [code, setCode] = useState(expression || "");
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [suggestionIndex, setSuggestionIndex] = useState(0);
  const [filteredVars, setFilteredVars] = useState(variables);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const lastWordRef = useRef("");

  // Extract the word being typed after the last space/operator
  const getCurrentWord = useCallback((text: string, cursorPos: number) => {
    const before = text.slice(0, cursorPos);
    const match = before.match(/[a-zA-Z_]+$/);
    return match ? match[0].toLowerCase() : "";
  }, []);

  const handleInput = (value: string) => {
    setCode(value);
    onChange(value);
    const cursorPos = textareaRef.current?.selectionStart ?? value.length;
    const word = getCurrentWord(value, cursorPos);
    lastWordRef.current = word;

    if (word.length >= 1) {
      const filtered = variables.filter((v) => v.name.toLowerCase().startsWith(word.toLowerCase()));
      setFilteredVars(filtered);
      setShowSuggestions(filtered.length > 0 && !filtered.some((v) => v.name.toLowerCase() === word.toLowerCase()));
      setSuggestionIndex(0);
    } else {
      setShowSuggestions(false);
    }
  };

  const acceptSuggestion = (varName: string) => {
    const cursorPos = textareaRef.current?.selectionStart ?? code.length;
    const before = code.slice(0, cursorPos);
    const after = code.slice(cursorPos);
    const wordLen = lastWordRef.current.length;
    const newBefore = before.slice(0, before.length - wordLen) + varName;
    const newCode = newBefore + after;
    setCode(newCode);
    onChange(newCode);
    setShowSuggestions(false);
    textareaRef.current?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showSuggestions) return;
    if (e.key === "Tab" || e.key === "Enter") {
      e.preventDefault();
      if (filteredVars[suggestionIndex]) acceptSuggestion(filteredVars[suggestionIndex].name);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setSuggestionIndex((i) => Math.min(i + 1, filteredVars.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSuggestionIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Escape") {
      setShowSuggestions(false);
    }
  };

  // Insert variable chip on click
  const insertVariable = (name: string) => {
    const newCode = code + (code.length > 0 && !code.endsWith(" ") ? " " : "") + name + " ";
    setCode(newCode);
    onChange(newCode);
    textareaRef.current?.focus();
  };

  // Safe expression evaluator for preview
  const evaluatePreview = useCallback((expr: string, vars: Record<string, number>): number | null => {
    try {
      const safeExpr = expr.replace(/\b(clamp|min|max|abs|round|floor|ceil|sqrt|pow|log|sin|cos)\b/g, "Math.$1");
      const varNames = Object.keys(vars);
      const fn = new Function(...varNames, `"use strict"; return (${safeExpr});`);
      return Number(fn(...varNames.map((k) => vars[k])));
    } catch { return null; }
  }, []);

  // Compute previews from images
  const computedPreviews = previews.length > 0 ? previews : (() => {
    if (!code.trim()) return [];
    // Mock preview with sample ISO values
    const sampleVals = [100, 400, 1600, 6400];
    return sampleVals.map((iso) => {
      const result = evaluatePreview(code, { iso, aperture: 4, shutter: 125, focal_length: 50, ev: 12, filename: 0 });
      return { label: `ISO ${iso}`, result: result ?? NaN };
    }).filter((p) => !isNaN(p.result));
  })();

  return (
    <div className="expr-box" style={{
      margin: "0 16px 12px", padding: "8px 12px",
      background: "rgba(176,132,244,0.04)", border: "1px solid rgba(176,132,244,0.15)",
      borderRadius: "var(--radiusLarge)", position: "relative",
    }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
        <span style={{ fontSize: "var(--fontSizeCaption1)", fontWeight: 600, color: "var(--expressionFg)" }}>
          &#120001; Expression
        </span>
        <span style={{ fontSize: "9px", color: "var(--neutralFg4)" }}>Esc to close</span>
      </div>

      {/* Variable chips */}
      {variables.length > 0 && (
        <div style={{ marginBottom: "6px", display: "flex", gap: "4px", flexWrap: "wrap" }}>
          {variables.map((v) => (
            <span key={v.name} title={`${v.description}${v.example ? ` (e.g. ${v.example})` : ""}`}
              style={{ fontSize: "9px", padding: "1px 6px", background: "rgba(176,132,244,0.1)", color: "var(--expressionFg)", borderRadius: "var(--radiusSmall)", cursor: "pointer", fontFamily: "var(--fontFamilyMono)" }}
              onClick={() => insertVariable(v.name)}>
              {v.name}
            </span>
          ))}
        </div>
      )}

      {/* Autocomplete dropdown */}
      {showSuggestions && filteredVars.length > 0 && (
        <div style={{
          position: "absolute", top: "100%", left: 12, right: 12, zIndex: 10,
          background: "var(--neutralBg2)", border: "1px solid var(--neutralStroke1)",
          borderRadius: "var(--radiusMedium)", boxShadow: "var(--shadow4)", maxHeight: 120, overflowY: "auto",
        }}>
          {filteredVars.map((v, i) => (
            <div key={v.name}
              style={{
                padding: "4px 10px", fontSize: "11px", fontFamily: "var(--fontFamilyMono)",
                background: i === suggestionIndex ? "var(--neutralBg3)" : "transparent",
                cursor: "pointer", display: "flex", justifyContent: "space-between",
              }}
              onClick={() => acceptSuggestion(v.name)}>
              <span style={{ color: "var(--expressionFg)" }}>{v.name}</span>
              <span style={{ color: "var(--neutralFg4)", fontSize: "9px" }}>{v.description}</span>
            </div>
          ))}
        </div>
      )}

      <textarea ref={textareaRef}
        value={code} onChange={(e) => handleInput(e.target.value)} onKeyDown={handleKeyDown}
        placeholder="e.g. clamp(iso / 12800, 0, 1)"
        rows={2}
        style={{ width: "100%", background: "transparent", border: "none", color: "var(--neutralFg1)", fontFamily: "var(--fontFamilyMono)", fontSize: "var(--fontSizeBody1)", resize: "vertical", outline: "none" }}
      />

      {/* Preview results */}
      {computedPreviews.length > 0 && (
        <div style={{ marginTop: "6px", display: "flex", gap: "12px", flexWrap: "wrap" }}>
          {computedPreviews.map((p) => (
            <span key={p.label} style={{ fontSize: "9px", color: "var(--neutralFg4)", fontFamily: "var(--fontFamilyMono)" }}>
              {p.label}&rarr;{p.result.toFixed(2)}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
