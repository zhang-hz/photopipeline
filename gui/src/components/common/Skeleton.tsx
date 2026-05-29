import "./Skeleton.css";

interface SkeletonProps {
  width?: number | string;
  height?: number;
  variant?: "text" | "rect" | "circle";
  count?: number;
  style?: React.CSSProperties;
}

export function Skeleton({ width = "100%", height = 16, variant = "text", count = 1, style }: SkeletonProps) {
  const items = Array.from({ length: count }, (_, i) => i);

  return (
    <>
      {items.map((i) => (
        <div
          key={i}
          className={`skel skel--${variant}`}
          style={{
            width: typeof width === "number" ? width : width,
            height,
            borderRadius: variant === "circle" ? "50%" : variant === "text" ? "var(--radiusMedium)" : "var(--radiusLarge)",
            marginBottom: i < count - 1 ? 8 : 0,
            ...style,
          }}
        />
      ))}
    </>
  );
}
