interface DAGEdgeProps {
  from: { x: number; y: number };
  to: { x: number; y: number };
  selected?: boolean;
}

export function DAGEdge({ from, to, selected }: DAGEdgeProps) {
  const dx = Math.abs(to.x - from.x) * 0.5;
  const d = `M ${from.x} ${from.y} C ${from.x + dx} ${from.y}, ${to.x - dx} ${to.y}, ${to.x} ${to.y}`;

  return (
    <g>
      {/* Wide invisible path for hit-testing */}
      <path d={d} fill="none" stroke="transparent" strokeWidth={12} style={{ pointerEvents: "stroke" }} />
      {/* Visible path */}
      <path
        d={d}
        fill="none"
        stroke={selected ? "var(--brandFg1)" : "var(--brandFg1)"}
        strokeWidth={selected ? 3 : 2}
        opacity={selected ? 1 : 0.55}
        style={{ pointerEvents: "none" }}
      />
    </g>
  );
}
