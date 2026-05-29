import { usePluginStore } from "../../stores/usePluginStore";
import { PLUGIN_COLORS } from "../../data/mockPlugins";

interface PluginHeaderProps {
  pluginId: string;
}

export function PluginHeader({ pluginId }: PluginHeaderProps) {
  const plugins = usePluginStore((s) => s.plugins);
  const plugin = plugins.find((p) => p.id === pluginId);
  if (!plugin) return null;

  const shortId = plugin.id.replace("photopipeline.plugins.", "");
  const color = PLUGIN_COLORS[shortId] ?? "var(--brandFg1)";

  return (
    <div style={{ flexShrink: 0 }}>
      <div style={{ display: "flex", gap: "12px", padding: "16px", alignItems: "flex-start" }}>
        <div style={{
          width: 40, height: 40, display: "flex", alignItems: "center", justifyContent: "center",
          background: `${color}20`, borderRadius: "var(--radiusMedium)", fontSize: "20px", flexShrink: 0,
        }}>
          &#9679;
        </div>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "baseline", gap: "8px" }}>
            <span style={{ fontSize: "var(--fontSizeHeading)", fontWeight: 600 }}>{plugin.name}</span>
            <span style={{ fontSize: "10px", color: "var(--neutralFg4)" }}>v{plugin.version}</span>
          </div>
          <div style={{ fontSize: "10px", color: "var(--neutralFg4)", fontFamily: "var(--fontFamilyMono)", marginTop: "2px" }}>
            {plugin.id}
          </div>
          <div style={{ display: "flex", gap: "4px", marginTop: "6px", flexWrap: "wrap" }}>
            <span style={{ fontSize: "9px", padding: "1px 6px", background: "var(--neutralBg3)", borderRadius: "var(--radiusSmall)", color: "var(--neutralFg2)" }}>
              {plugin.category}
            </span>
            {plugin.tags.slice(0, 3).map((t) => (
              <span key={t} style={{ fontSize: "9px", padding: "1px 6px", background: "var(--neutralBg3)", borderRadius: "var(--radiusSmall)", color: "var(--neutralFg4)" }}>
                {t}
              </span>
            ))}
          </div>
          <div style={{ fontSize: "10px", color: "var(--neutralFg4)", marginTop: "4px" }}>
            {plugin.requires_pixel_access ? "PixelProcessor" : "FormatProcessor"} &middot; RAM &ge; {plugin.min_ram_mb} MB
          </div>
        </div>
      </div>
      <div style={{ padding: "12px 16px", fontSize: "var(--fontSizeBody1)", color: "var(--neutralFg2)", lineHeight: 1.5, borderTop: "1px solid var(--neutralStroke1)" }}>
        {plugin.description}
      </div>
    </div>
  );
}
