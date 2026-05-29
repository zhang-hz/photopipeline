import "./OverrideDot.css";

export type DotState = "inherited" | "override" | "expression";

interface OverrideDotProps {
  state: DotState;
  canEdit?: boolean;
  onActivate?: () => void;
  onRestore?: () => void;
  onExpressionEdit?: () => void;
}

export function OverrideDot({ state, canEdit = true, onActivate, onRestore, onExpressionEdit }: OverrideDotProps) {
  const klass = `odot odot--${state}`;
  const title = state === "inherited" ? "Inherited — click to override"
    : state === "override" ? "Overridden — click to restore"
    : "Expression — double-click to edit";

  const handleClick = () => {
    if (!canEdit) return;
    if (state === "inherited") onActivate?.();
    else if (state === "override") onRestore?.();
  };
  const handleDblClick = () => {
    if (!canEdit) return;
    if (state === "override" || state === "expression") onExpressionEdit?.();
  };

  return (
    <span
      className={klass}
      title={title}
      onClick={handleClick}
      onDoubleClick={handleDblClick}
      role="button"
      aria-label={title}
    />
  );
}
