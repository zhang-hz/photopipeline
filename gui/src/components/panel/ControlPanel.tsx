import { useMemo, useEffect } from "react";
import { usePluginStore } from "../../stores/usePluginStore";
import { useOverrideStore } from "../../stores/useOverrideStore";
import { ParamSection, type SectionData } from "./ParamSection";
import { type DotState } from "../common/OverrideDot";
import { RemoveButton } from "./RemoveButton";
import { AuxView } from "./AuxView";
import type { MockParameterField, MockParameterSection } from "../../data/mockSchemas";

interface ControlPanelProps { nodeId: string; nodeLabel: string; pluginId: string; }

function fieldToWidget(field: MockParameterField, currentValue: unknown) {
  return {
    type: field.type === "integer" ? (field.style === "slider" ? "slider" : "number_input")
        : field.type === "float" ? (field.style === "slider" ? "slider" : "number_input")
        : field.type === "string" ? "text_input"
        : field.type === "boolean" ? "toggle"
        : field.type === "enum" ? "dropdown"
        : field.type === "file_path" ? "file_picker"
        : field.type === "color" ? "color_picker"
        : field.type === "coordinate" ? "coordinate_input"
        : field.type,
    paramId: field.id, label: field.label, description: field.description, value: currentValue ?? field.default,
    min: field.min, max: field.max, step: field.step, precision: field.precision, unit: field.unit,
    placeholder: field.placeholder, options: field.options, kind: field.kind, filters: field.filters,
  };
}

export function ControlPanel({ nodeId, nodeLabel, pluginId }: ControlPanelProps) {
  const fetchNodeSchema = usePluginStore((s) => s.fetchNodeSchema);
  const nodeSchemas = usePluginStore((s) => s.nodeSchemas);
  const scope = useOverrideStore((s) => s.scope);
  const setOverride = useOverrideStore((s) => s.setOverride);
  const clearOverride = useOverrideStore((s) => s.clearOverride);
  const setExpression = useOverrideStore((s) => s.setExpression);
  const getEffectiveValue = useOverrideStore((s) => s.getEffectiveValue);

  // Fetch schema on mount or when pluginId changes
  useEffect(() => {
    fetchNodeSchema(pluginId);
  }, [pluginId, fetchNodeSchema]);

  const schema = nodeSchemas.get(pluginId);
  const canEdit = scope !== "all";

  const sections: SectionData[] = useMemo(() => {
    if (!schema) return [];
    const paramSchema = schema.parameter_schema as any;
    const rawSections: MockParameterSection[] = paramSchema?.sections ?? [];

    return rawSections.map((s) => {
      const widgets = s.fields.map((f) => {
        const { value } = getEffectiveValue(nodeId, f.id);
        return fieldToWidget(f, value);
      });
      let overrideCount = 0;
      for (const f of s.fields) {
        if (getEffectiveValue(nodeId, f.id).isOverridden) overrideCount++;
      }
      return {
        id: s.id, label: s.label, description: s.description,
        collapsible: s.collapsible, defaultCollapsed: s.default_collapsed,
        widgets,
        overrideBadge: overrideCount > 0 ? ("overrides" as const) : ("inherited" as const),
        overrideCount,
      };
    });
  }, [schema, nodeId, getEffectiveValue]);

  const getDotState = (paramId: string): DotState => {
    const { isOverridden, isExpression } = getEffectiveValue(nodeId, paramId);
    if (isExpression) return "expression";
    if (isOverridden) return "override";
    return "inherited";
  };

  const auxViews: string[] = (schema?.gui_schema as any)?.aux_views ?? [];

  if (!schema) {
    return <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--neutralFg4)", fontSize: "12px" }}>加载 Schema 中...</div>;
  }

  return (
    <div style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column" }}>
      {sections.map((s) => (
        <ParamSection key={s.id} section={s} nodeId={nodeId} getDotState={getDotState}
          canEdit={canEdit}
          onChange={(pid, val) => setOverride(nodeId, pid, val)}
          onActivate={(pid) => setOverride(nodeId, pid, undefined)}
          onRestore={(pid) => clearOverride(nodeId, pid)}
          onExpressionEdit={(pid) => setExpression(nodeId, pid, "")} />
      ))}
      {auxViews.length > 0 && (
        <div style={{ padding: "0 16px 12px", display: "flex", flexDirection: "column", gap: "8px" }}>
          {auxViews.map((type: string) => <AuxView key={type} type={type} />)}
        </div>
      )}
      <RemoveButton nodeId={nodeId} nodeLabel={nodeLabel} />
    </div>
  );
}
