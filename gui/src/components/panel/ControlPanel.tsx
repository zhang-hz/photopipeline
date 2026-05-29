import { useMemo } from "react";
import { useOverrideStore } from "../../stores/useOverrideStore";
import { ParamSection, type SectionData } from "./ParamSection";
import { type DotState } from "../common/OverrideDot";
import { RemoveButton } from "./RemoveButton";
import { AuxView } from "./AuxView";
import { MOCK_SCHEMAS, type MockParameterField } from "../../data/mockSchemas";

interface ControlPanelProps {
  nodeId: string;
  nodeLabel: string;
  pluginId: string;
}

// Convert backend MockParameterField to PanelWidget for rendering
function fieldToWidget(field: MockParameterField, currentValue: unknown) {
  const base = {
    type: field.type,
    paramId: field.id,
    label: field.label,
    value: currentValue ?? field.default,
    description: field.description,
    min: field.min, max: field.max, step: field.step, precision: field.precision, unit: field.unit,
    placeholder: field.placeholder,
    options: field.options,
    kind: field.kind, filters: field.filters,
  };

  switch (field.type) {
    case "integer": return { ...base, type: field.style === "slider" ? "slider" : "number_input" };
    case "float": return { ...base, type: field.style === "slider" ? "slider" : "number_input" };
    case "string": return { ...base, type: "text_input" };
    case "boolean": return { ...base, type: "toggle", labelOn: field.label_true, labelOff: field.label_false };
    case "enum": return { ...base, type: "dropdown" };
    case "file_path": return { ...base, type: "file_picker" };
    case "color": return { ...base, type: "color_picker" };
    case "coordinate": return { ...base, type: "coordinate_input" };
    default: return base;
  }
}

function sectionsToUI(schema: typeof MOCK_SCHEMAS[string], nodeId: string): SectionData[] {
  return schema.sections.map((s) => {
    // Filter visible fields (respect conditions — simplified for now)
    const visibleFields = s.fields.filter((f) => {
      // Only skip advanced fields in collapsed view
      return true;
    });

    const widgets = visibleFields.map((f) => {
      // Get current effective value from override store
      const { value } = useOverrideStore.getState().getEffectiveValue(nodeId, f.id);
      return fieldToWidget(f, value);
    });

    // Compute override summary
    let overrideCount = 0;
    for (const f of visibleFields) {
      const { isOverridden } = useOverrideStore.getState().getEffectiveValue(nodeId, f.id);
      if (isOverridden) overrideCount++;
    }

    return {
      id: s.id,
      label: s.label,
      description: s.description,
      collapsible: s.collapsible,
      defaultCollapsed: s.default_collapsed,
      widgets,
      overrideBadge: overrideCount > 0 ? ("overrides" as const) : ("inherited" as const),
      overrideCount,
    };
  });
}

export function ControlPanel({ nodeId, nodeLabel, pluginId }: ControlPanelProps) {
  const scope = useOverrideStore((s) => s.scope);
  const setOverride = useOverrideStore((s) => s.setOverride);
  const clearOverride = useOverrideStore((s) => s.clearOverride);
  const setExpression = useOverrideStore((s) => s.setExpression);
  const getEffectiveValue = useOverrideStore((s) => s.getEffectiveValue);

  const schema = MOCK_SCHEMAS[pluginId];
  const sections = useMemo(() => {
    if (!schema) return [];
    return sectionsToUI(schema, nodeId);
  }, [schema, nodeId]);

  const canEdit = scope !== "all";
  const hasExpression = false; // TODO: wire up

  const getDotState = (paramId: string): DotState => {
    const { isOverridden, isExpression } = getEffectiveValue(nodeId, paramId);
    if (isExpression) return "expression";
    if (isOverridden) return "override";
    return "inherited";
  };

  const handleChange = (paramId: string, value: unknown) => {
    setOverride(nodeId, paramId, value);
  };

  if (!schema) {
    return (
      <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--neutralFg4)", fontSize: "12px" }}>
        Schema not available
      </div>
    );
  }

  return (
    <div style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column" }}>
      {sections.map((s) => (
        <ParamSection
          key={s.id}
          section={s}
          nodeId={nodeId}
          getDotState={getDotState}
          canEdit={canEdit}
          onChange={handleChange}
          onActivate={(pid) => setOverride(nodeId, pid, undefined)}
          onRestore={(pid) => clearOverride(nodeId, pid)}
          onExpressionEdit={(pid) => setExpression(nodeId, pid, "")}
        />
      ))}

      {/* Auxiliary Views */}
      {schema.aux_views.length > 0 && (
        <div style={{ padding: "0 16px 12px", display: "flex", flexDirection: "column", gap: "8px" }}>
          {schema.aux_views.map((type) => (
            <AuxView key={type} type={type} />
          ))}
        </div>
      )}

      <RemoveButton nodeId={nodeId} nodeLabel={nodeLabel} />
    </div>
  );
}
