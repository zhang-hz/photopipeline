export interface PluginEntry {
  id: string;
  name: string;
  version: string;
  category: string;
  description: string;
  tags: string[];
  requires_pixel_access: boolean;
  requires_network: boolean;
  requires_filesystem: boolean;
  min_ram_mb: number;
}

export interface NodeSchemaResponse {
  plugin_id: string;
  name: string;
  version: string;
  category: string;
  description: string;
  parameter_schema: Record<string, unknown>;
  gui_schema: Record<string, unknown>;
}
