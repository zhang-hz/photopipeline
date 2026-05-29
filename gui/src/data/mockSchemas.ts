// Mock NodeSchemaResponse data matching backend ParameterSchema format
// Types: string, integer, float, boolean, enum, file_path, color, coordinate, slider

export interface MockParameterField {
  id: string;
  label: string;
  description?: string;
  type: string;
  default: unknown;
  required: boolean;
  advanced: boolean;
  allow_override: boolean;
  supports_expression: boolean;
  // type-specific
  min?: number; max?: number; step?: number; precision?: number; unit?: string;
  placeholder?: string; max_length?: number;
  options?: { value: string; label: string; description?: string; recommended?: boolean }[];
  label_true?: string; label_false?: string;
  kind?: string; filters?: [string, string][];
  style?: string;
}

export interface MockParameterSection {
  id: string;
  label: string;
  description?: string;
  collapsible: boolean;
  default_collapsed: boolean;
  fields: MockParameterField[];
}

export interface MockNodeSchema {
  plugin_id: string;
  name: string;
  version: string;
  category: string;
  description: string;
  sections: MockParameterSection[];
  aux_views: string[];
  preview: string;
  min_panel_width: number;
}

// 对照后端 crates/plugins/src/ 的 PARAMETER_SCHEMA 和 GUI_SCHEMA
export const MOCK_SCHEMAS: Record<string, MockNodeSchema> = {
  "photopipeline.plugins.raw_input": {
    plugin_id: "photopipeline.plugins.raw_input",
    name: "RAW Input", version: "1.0.0", category: "Input",
    description: "Read RAW camera files (ARW, CR2, CR3, NEF, DNG, RAF, ORF, RW2, PEF)",
    sections: [
      { id: "raw_format", label: "RAW Format", collapsible: false, default_collapsed: false, fields: [
        { id: "raw_mode", label: "Decode Mode", description: "How to process the RAW file", type: "enum", default: "auto", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "auto", label: "Auto", description: "Detect from file and use best method", recommended: true },
            { value: "dcraw", label: "dcraw", description: "Use dcraw for raw conversion" },
            { value: "libraw", label: "LibRaw", description: "Use LibRaw via FFI (when available)" }] },
      ]},
      { id: "output", label: "Output", collapsible: false, default_collapsed: false, fields: [
        { id: "output_format", label: "Pixel Format", description: "Output pixel format after decoding", type: "enum", default: "u16", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "u16", label: "16-bit", description: "Standard 16-bit integer", recommended: true },
            { value: "f32", label: "32-bit float", description: "Floating-point for HDR" }] },
        { id: "half_size", label: "Half Size", description: "Decode at half resolution for fast preview", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false, label_true: "Half", label_false: "Full" },
        { id: "apply_white_balance", label: "White Balance", description: "Apply camera white balance to output", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false, label_true: "Apply", label_false: "As-Shot" },
      ]},
      { id: "dcraw_options", label: "dcraw Options", collapsible: true, default_collapsed: true, fields: [
        { id: "dcraw_path", label: "dcraw Path", description: "Path to dcraw binary", type: "string", default: "dcraw", required: false, advanced: true, allow_override: true, supports_expression: false, placeholder: "/usr/bin/dcraw" },
        { id: "dcraw_extra_args", label: "Extra Args", description: "Additional dcraw command-line arguments", type: "string", default: "", required: false, advanced: true, allow_override: true, supports_expression: false, placeholder: "-H 2" },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.transform": {
    plugin_id: "photopipeline.plugins.transform",
    name: "Transform", version: "1.0.0", category: "Transform",
    description: "Resize, rotate, and crop images with configurable filters",
    sections: [
      { id: "resize", label: "Resize", collapsible: false, default_collapsed: false, fields: [
        { id: "width", label: "Width", description: "Target width in pixels, 0 = scale proportionally", type: "integer", default: 1920, min: 1, max: 65535, step: 1, unit: "px", required: true, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "height", label: "Height", description: "Target height in pixels, 0 = scale proportionally", type: "integer", default: 1080, min: 1, max: 65535, step: 1, unit: "px", required: true, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "keep_aspect", label: "Keep Aspect", description: "Maintain original aspect ratio when resizing", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false, label_true: "Keep", label_false: "Ignore" },
        { id: "filter", label: "Filter", description: "Resampling filter for quality/speed trade-off", type: "enum", default: "lanczos3", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "lanczos3", label: "Lanczos3", description: "Highest quality, slower", recommended: true },
            { value: "bilinear", label: "Bilinear", description: "Fast, lower quality" },
            { value: "nearest", label: "Nearest Neighbor", description: "No interpolation" }] },
        { id: "fit_mode", label: "Fit Mode", description: "How to fit the image into target dimensions", type: "enum", default: "fit", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "fit", label: "Fit", description: "Scale to fit, keep aspect", recommended: true },
            { value: "fill", label: "Fill", description: "Scale to fill, may crop" },
            { value: "crop", label: "Crop", description: "Center crop to dimensions" }] },
      ]},
      { id: "rotation", label: "Rotation", collapsible: false, default_collapsed: false, fields: [
        { id: "angle", label: "Angle", description: "Rotation angle in degrees", type: "float", default: 0.0, min: -360, max: 360, step: 0.1, precision: 1, unit: "°", required: false, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "flip_horizontal", label: "Flip H", description: "Flip horizontally (mirror)", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "flip_vertical", label: "Flip V", description: "Flip vertically", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "auto_orient", label: "Auto Orient", description: "Auto-rotate based on EXIF orientation tag", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "background_color", label: "Background", description: "Fill color for exposed areas after rotation", type: "color", default: "#000000", required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
      { id: "crop", label: "Crop", collapsible: true, default_collapsed: true, fields: [
        { id: "crop_enabled", label: "Enable Crop", description: "Enable crop operation", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "crop_x", label: "X", description: "Crop start X coordinate", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
        { id: "crop_y", label: "Y", description: "Crop start Y coordinate", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
        { id: "crop_width", label: "Width", description: "Crop width (0 = no limit)", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
        { id: "crop_height", label: "Height", description: "Crop height (0 = no limit)", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
      ]},
    ], aux_views: ["histogram"], preview: "before_after", min_panel_width: 340,
  },
};
