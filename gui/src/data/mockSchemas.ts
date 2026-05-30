/* eslint-disable @typescript-eslint/no-explicit-any */
// 14 个插件的完整 ParameterSchema — 严格对照后端 crates/plugins/src/ 源码

export interface MockParameterField {
  id: string; label: string; description?: string; type: string; default: any; required: boolean; advanced: boolean; allow_override: boolean; supports_expression: boolean;
  min?: number; max?: number; step?: number; precision?: number; unit?: string; placeholder?: string; max_length?: number;
  options?: { value: string; label: string; description?: string; recommended?: boolean }[]; label_true?: string; label_false?: string;
  kind?: string; filters?: [string, string][]; style?: string;
}

export interface MockParameterSection { id: string; label: string; description?: string; collapsible: boolean; default_collapsed: boolean; fields: MockParameterField[]; }

export interface MockNodeSchema {
  plugin_id: string; name: string; version: string; category: string; description: string; sections: MockParameterSection[]; aux_views: string[]; preview: string; min_panel_width: number;
}

export const MOCK_SCHEMAS: Record<string, MockNodeSchema> = {
  "photopipeline.plugins.raw_input": {
    plugin_id: "photopipeline.plugins.raw_input", name: "RAW 输入", version: "1.0.0", category: "Input",
    description: "读取 RAW 相机文件 (ARW, CR2, CR3, NEF, DNG, RAF, ORF, RW2, PEF)",
    sections: [
      { id: "raw_format", label: "RAW 格式", collapsible: false, default_collapsed: false, fields: [
        { id: "raw_mode", label: "解码模式", description: "处理 RAW 文件的方式", type: "enum", default: "auto", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "auto", label: "自动", description: "检测文件并使用最佳方式", recommended: true }, { value: "dcraw", label: "dcraw" }, { value: "libraw", label: "LibRaw (FFI)" }] },
      ]},
      { id: "output", label: "输出", collapsible: false, default_collapsed: false, fields: [
        { id: "output_format", label: "像素格式", description: "解码后的像素格式", type: "enum", default: "u16", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "u16", label: "16-bit 整数", recommended: true }, { value: "f32", label: "32-bit 浮点 (HDR)" }] },
        { id: "half_size", label: "半分辨率", description: "以半分辨率解码用于快速预览", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "apply_white_balance", label: "应用白平衡", description: "将相机白平衡应用到输出", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.transform": {
    plugin_id: "photopipeline.plugins.transform", name: "几何变换", version: "1.0.0", category: "Transform",
    description: "缩放、旋转和裁剪图像，支持多种重采样滤镜",
    sections: [
      { id: "resize", label: "缩放", collapsible: false, default_collapsed: false, fields: [
        { id: "width", label: "宽度", description: "目标宽度 (px)", type: "integer", default: 1920, min: 1, max: 65535, step: 1, unit: "px", required: true, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "height", label: "高度", description: "目标高度 (px)", type: "integer", default: 1080, min: 1, max: 65535, step: 1, unit: "px", required: true, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "keep_aspect", label: "保持比例", description: "保持原始宽高比", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "filter", label: "滤镜", description: "重采样滤镜", type: "enum", default: "lanczos3", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "lanczos3", label: "Lanczos3", description: "最高质量", recommended: true }, { value: "bilinear", label: "Bilinear" }, { value: "nearest", label: "Nearest Neighbor" }] },
        { id: "fit_mode", label: "缩放模式", type: "enum", default: "fit", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "fit", label: "适应", recommended: true }, { value: "fill", label: "填充" }, { value: "crop", label: "裁剪" }] },
      ]},
      { id: "rotation", label: "旋转", collapsible: false, default_collapsed: false, fields: [
        { id: "angle", label: "角度", description: "旋转角度", type: "float", default: 0.0, min: -360, max: 360, step: 0.1, precision: 1, unit: "°", required: false, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "flip_horizontal", label: "水平翻转", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "flip_vertical", label: "垂直翻转", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "background_color", label: "背景色", description: "旋转后暴露区域的填充色", type: "color", default: "#000000", required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
      { id: "crop", label: "裁剪", collapsible: true, default_collapsed: true, fields: [
        { id: "crop_enabled", label: "启用裁剪", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "crop_x", label: "X 起点", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
        { id: "crop_y", label: "Y 起点", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
        { id: "crop_width", label: "裁剪宽度", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
        { id: "crop_height", label: "裁剪高度", type: "integer", default: 0, min: 0, max: 65535, step: 1, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: true },
      ]},
    ], aux_views: ["histogram"], preview: "before_after", min_panel_width: 340,
  },

  "photopipeline.plugins.colorspace": {
    plugin_id: "photopipeline.plugins.colorspace", name: "色彩空间", version: "1.0.0", category: "Color",
    description: "色彩空间转换，支持 ICC Profile 和渲染意图",
    sections: [
      { id: "conversion", label: "转换", collapsible: false, default_collapsed: false, fields: [
        { id: "source", label: "源色彩空间", type: "enum", default: "sRGB", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "sRGB", label: "sRGB", recommended: true }, { value: "Adobe RGB", label: "Adobe RGB" }, { value: "Display P3", label: "Display P3" }, { value: "ProPhoto RGB", label: "ProPhoto RGB" }, { value: "BT.2020", label: "BT.2020 (HDR)" }] },
        { id: "target", label: "目标色彩空间", type: "enum", default: "Display P3", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "sRGB", label: "sRGB" }, { value: "Display P3", label: "Display P3", recommended: true }, { value: "Adobe RGB", label: "Adobe RGB" }, { value: "BT.2020", label: "BT.2020 (HDR)" }] },
        { id: "intent", label: "渲染意图", type: "enum", default: "Relative", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "Relative", label: "相对色度", recommended: true }, { value: "Perceptual", label: "感知" }, { value: "Saturation", label: "饱和度" }, { value: "Absolute", label: "绝对色度" }] },
      ]},
      { id: "rendering", label: "渲染", collapsible: false, default_collapsed: false, fields: [
        { id: "black_point_compensation", label: "黑点补偿", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "gamut_clipping", label: "色域裁切", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
      { id: "icc_profile", label: "ICC Profile", collapsible: true, default_collapsed: true, fields: [
        { id: "icc_profile_path", label: "ICC 文件路径", type: "file_path", default: "", required: false, advanced: true, allow_override: true, supports_expression: false, kind: "file", filters: [["ICC Profiles", "*.icc;*.icm"]] },
        { id: "embed_icc", label: "嵌入 ICC", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
    ], aux_views: ["histogram", "gamut_diagram"], preview: "before_after", min_panel_width: 340,
  },

  "photopipeline.plugins.lut3d": {
    plugin_id: "photopipeline.plugins.lut3d", name: "3D LUT", version: "1.0.0", category: "Color",
    description: "应用 3D LUT 进行色彩分级和电影感模拟", sections: [
      { id: "lut_file", label: "LUT 文件", collapsible: false, default_collapsed: false, fields: [
        { id: "lut_path", label: "LUT 文件路径", type: "file_path", default: "", required: true, advanced: false, allow_override: true, supports_expression: false, kind: "file", filters: [["LUT Files", "*.cube;*.3dl"]] },
        { id: "lut_format", label: "格式", type: "enum", default: "auto", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "auto", label: "自动", recommended: true }, { value: "cube", label: "Cube" }, { value: "3dl", label: "3DL" }] },
      ]},
      { id: "lut_transform", label: "LUT 变换", collapsible: false, default_collapsed: false, fields: [
        { id: "intensity", label: "强度", description: "LUT 混合强度", type: "float", default: 1.0, min: 0, max: 1, step: 0.01, precision: 2, unit: "%", required: true, advanced: false, allow_override: true, supports_expression: false, style: "slider" },
        { id: "input_color_space", label: "输入色彩空间", type: "enum", default: "auto", required: false, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "auto", label: "自动", recommended: true }, { value: "sRGB", label: "sRGB" }, { value: "Rec.709", label: "Rec.709" }] },
        { id: "output_color_space", label: "输出色彩空间", type: "enum", default: "auto", required: false, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "auto", label: "自动", recommended: true }, { value: "sRGB", label: "sRGB" }, { value: "Rec.709", label: "Rec.709" }] },
      ]},
    ], aux_views: ["histogram", "vectorscope"], preview: "before_after", min_panel_width: 320,
  },

  "photopipeline.plugins.lens_correct": {
    plugin_id: "photopipeline.plugins.lens_correct", name: "镜头校正", version: "1.0.0", category: "Enhance",
    description: "校正镜头畸变、色差和暗角 (LensFun)", sections: [
      { id: "lens_detection", label: "镜头检测", collapsible: false, default_collapsed: false, fields: [
        { id: "auto_detect", label: "自动检测", description: "从 EXIF 自动检测镜头", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "camera_make", label: "相机品牌", type: "string", default: "", required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "lens_name", label: "镜头名称", type: "string", default: "", required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
      { id: "corrections", label: "校正", collapsible: false, default_collapsed: false, fields: [
        { id: "distortion", label: "畸变校正", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "tca", label: "色差校正 (TCA)", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "vignetting", label: "暗角校正", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "geometry_scale", label: "几何校正强度", type: "float", default: 1.0, min: 0.5, max: 2.0, step: 0.1, precision: 1, required: false, advanced: false, allow_override: true, supports_expression: false, style: "slider" },
      ]},
    ], aux_views: ["histogram", "status_text"], preview: "before_after", min_panel_width: 340,
  },

  "photopipeline.plugins.ai_denoise": {
    plugin_id: "photopipeline.plugins.ai_denoise", name: "AI 降噪", version: "1.0.0", category: "Enhance",
    description: "基于 ONNX 模型的 AI 降噪", sections: [
      { id: "model", label: "模型", collapsible: false, default_collapsed: false, fields: [
        { id: "model", label: "模型选择", type: "enum", default: "standard_v2", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "lightweight_v1", label: "轻量 v1", description: "快速" }, { value: "standard_v2", label: "标准 v2", description: "平衡", recommended: true }, { value: "high_quality_v2", label: "高质 v2", description: "最佳质量，较慢" }] },
        { id: "model_source", label: "模型来源", type: "enum", default: "huggingface", required: false, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "huggingface", label: "HuggingFace", recommended: true }, { value: "local", label: "本地" }] },
        { id: "model_path", label: "本地模型路径", type: "file_path", default: "", required: false, advanced: false, allow_override: true, supports_expression: false, kind: "file" },
      ]},
      { id: "strength", label: "强度", collapsible: false, default_collapsed: false, fields: [
        { id: "strength", label: "总体强度", type: "float", default: 0.5, min: 0, max: 1, step: 0.01, precision: 2, required: true, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "luma_strength", label: "亮度降噪", type: "float", default: 0.5, min: 0, max: 1, step: 0.01, precision: 2, required: false, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
        { id: "chroma_strength", label: "色度降噪", type: "float", default: 0.5, min: 0, max: 1, step: 0.01, precision: 2, required: false, advanced: false, allow_override: true, supports_expression: true, style: "slider" },
      ]},
      { id: "hardware", label: "硬件", collapsible: true, default_collapsed: true, fields: [
        { id: "backend", label: "后端", type: "enum", default: "auto", required: false, advanced: true, allow_override: true, supports_expression: false,
          options: [{ value: "auto", label: "自动", recommended: true }, { value: "cpu", label: "CPU" }, { value: "cuda", label: "CUDA (GPU)" }, { value: "coreml", label: "CoreML (Apple)" }] },
        { id: "tile_size", label: "分块大小", type: "integer", default: 1024, min: 256, max: 4096, step: 64, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: false },
        { id: "tile_overlap", label: "分块重叠", type: "integer", default: 64, min: 0, max: 512, step: 8, unit: "px", required: false, advanced: true, allow_override: true, supports_expression: false },
      ]},
    ], aux_views: ["histogram", "progress_bar", "status_text"], preview: "before_after", min_panel_width: 360,
  },

  "photopipeline.plugins.exif_rw": {
    plugin_id: "photopipeline.plugins.exif_rw", name: "EXIF 读写", version: "1.0.0", category: "Metadata",
    description: "读写 EXIF、XMP、IPTC 和 GPS 元数据", sections: [
      { id: "read_options", label: "读取选项", collapsible: false, default_collapsed: false, fields: [
        { id: "extract_exif", label: "提取 EXIF", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "extract_xmp", label: "提取 XMP", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "extract_iptc", label: "提取 IPTC", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
      { id: "write_options", label: "写入选项", collapsible: false, default_collapsed: false, fields: [
        { id: "preserve_source", label: "保留源元数据", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "copyright", label: "版权信息", type: "string", default: "", required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "artist", label: "作者/摄影师", type: "string", default: "", required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "description", label: "图像描述", type: "string", default: "", required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "keywords", label: "关键词", description: "逗号分隔", type: "string", default: "", required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "rating", label: "评分", type: "integer", default: 0, min: 0, max: 5, step: 1, required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.gps_set": {
    plugin_id: "photopipeline.plugins.gps_set", name: "GPS 坐标", version: "1.0.0", category: "Metadata",
    description: "设置 GPS 坐标，支持手动输入和 GPX 轨迹插值", sections: [
      { id: "source", label: "来源", collapsible: false, default_collapsed: false, fields: [
        { id: "mode", label: "GPS 模式", type: "enum", default: "manual", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "exif_preserve", label: "保留 EXIF" }, { value: "manual", label: "手动输入", recommended: true }, { value: "gpx", label: "GPX 轨迹" }, { value: "clear", label: "清除 GPS" }] },
      ]},
      { id: "manual_coords", label: "手动坐标", collapsible: false, default_collapsed: false, fields: [
        { id: "latitude", label: "纬度", type: "float", default: 0.0, min: -90, max: 90, step: 0.000001, precision: 6, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "longitude", label: "经度", type: "float", default: 0.0, min: -180, max: 180, step: 0.000001, precision: 6, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "altitude", label: "海拔", type: "float", default: 0.0, min: -500, max: 9000, step: 0.1, precision: 1, unit: "m", required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
      { id: "gpx_options", label: "GPX 选项", collapsible: true, default_collapsed: true, fields: [
        { id: "gpx_file", label: "GPX 轨迹文件", type: "file_path", default: "", required: false, advanced: false, allow_override: true, supports_expression: false, kind: "file", filters: [["GPX Files", "*.gpx"]] },
        { id: "time_offset", label: "时间偏移", type: "integer", default: 0, min: -86400, max: 86400, step: 1, unit: "s", required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
    ], aux_views: ["map"], preview: "none", min_panel_width: 340,
  },

  "photopipeline.plugins.time_shift": {
    plugin_id: "photopipeline.plugins.time_shift", name: "时间偏移", version: "1.0.0", category: "Metadata",
    description: "调整拍摄时间，支持时区转换", sections: [
      { id: "time_shift", label: "时间调整", collapsible: false, default_collapsed: false, fields: [
        { id: "hours", label: "小时", type: "integer", default: 0, min: -23, max: 23, step: 1, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "minutes", label: "分钟", type: "integer", default: 0, min: -59, max: 59, step: 1, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "seconds", label: "秒", type: "integer", default: 0, min: -59, max: 59, step: 1, required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
      { id: "timezone", label: "时区", collapsible: false, default_collapsed: false, fields: [
        { id: "timezone_from", label: "源时区", type: "enum", default: "UTC", required: false, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "UTC", label: "UTC" }, { value: "local", label: "本地" }, { value: "Asia/Shanghai", label: "亚洲/上海" }, { value: "Asia/Tokyo", label: "亚洲/东京" }, { value: "America/New_York", label: "美洲/纽约" }] },
        { id: "timezone_to", label: "目标时区", type: "enum", default: "local", required: false, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "local", label: "本地" }, { value: "UTC", label: "UTC" }, { value: "Asia/Shanghai", label: "亚洲/上海" }, { value: "Asia/Tokyo", label: "亚洲/东京" }] },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.heif_encoder": {
    plugin_id: "photopipeline.plugins.heif_encoder", name: "HEIF 编码器", version: "1.0.0", category: "Export",
    description: "编码为 HEIF/HEIC 格式，支持 10-bit HDR", sections: [
      { id: "quality", label: "质量", collapsible: false, default_collapsed: false, fields: [
        { id: "quality", label: "质量", type: "float", default: 95, min: 0, max: 100, step: 1, precision: 0, required: true, advanced: false, allow_override: true, supports_expression: false, style: "slider" },
        { id: "lossless", label: "无损", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "bit_depth", label: "位深度", type: "enum", default: "10", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "10", label: "10-bit", recommended: true }, { value: "8", label: "8-bit" }, { value: "12", label: "12-bit" }] },
      ]},
      { id: "advanced", label: "高级", collapsible: true, default_collapsed: true, fields: [
        { id: "encoder", label: "编码器", type: "enum", default: "x265", required: false, advanced: true, allow_override: true, supports_expression: false,
          options: [{ value: "x265", label: "x265", recommended: true }, { value: "x264", label: "x264" }] },
        { id: "preset", label: "预设", type: "enum", default: "veryslow", required: false, advanced: true, allow_override: true, supports_expression: false,
          options: [{ value: "ultrafast", label: "超快" }, { value: "medium", label: "中等" }, { value: "veryslow", label: "极慢", recommended: true }] },
        { id: "chroma_subsampling", label: "色度采样", type: "enum", default: "444", required: false, advanced: true, allow_override: true, supports_expression: false,
          options: [{ value: "444", label: "4:4:4", recommended: true }, { value: "422", label: "4:2:2" }, { value: "420", label: "4:2:0" }] },
        { id: "tune", label: "优化", type: "enum", default: "ssim", required: false, advanced: true, allow_override: true, supports_expression: false,
          options: [{ value: "ssim", label: "SSIM", recommended: true }, { value: "psnr", label: "PSNR" }, { value: "grain", label: "颗粒" }] },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.jxl_encoder": {
    plugin_id: "photopipeline.plugins.jxl_encoder", name: "JPEG XL 编码器", version: "1.0.0", category: "Export",
    description: "编码为 JPEG XL 格式，支持 16-bit HDR", sections: [
      { id: "quality", label: "质量", collapsible: false, default_collapsed: false, fields: [
        { id: "quality", label: "质量", type: "float", default: 90, min: 0, max: 100, step: 1, precision: 0, required: true, advanced: false, allow_override: true, supports_expression: false, style: "slider" },
        { id: "lossless", label: "无损", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "bit_depth", label: "位深度", type: "enum", default: "16", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "16", label: "16-bit", recommended: true }, { value: "8", label: "8-bit" }] },
      ]},
      { id: "advanced", label: "高级", collapsible: true, default_collapsed: true, fields: [
        { id: "effort", label: "编码努力", type: "integer", default: 7, min: 1, max: 9, step: 1, required: false, advanced: true, allow_override: true, supports_expression: false },
        { id: "modular", label: "模块化模式", type: "boolean", default: false, required: false, advanced: true, allow_override: true, supports_expression: false },
        { id: "progressive", label: "渐进式", type: "boolean", default: true, required: false, advanced: true, allow_override: true, supports_expression: false },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.avif_encoder": {
    plugin_id: "photopipeline.plugins.avif_encoder", name: "AVIF 编码器", version: "1.0.0", category: "Export",
    description: "编码为 AVIF 格式，支持 10-bit HDR", sections: [
      { id: "quality", label: "质量", collapsible: false, default_collapsed: false, fields: [
        { id: "quality", label: "质量", type: "float", default: 85, min: 0, max: 100, step: 1, precision: 0, required: true, advanced: false, allow_override: true, supports_expression: false, style: "slider" },
        { id: "speed", label: "速度", type: "integer", default: 6, min: 0, max: 10, step: 1, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "bit_depth", label: "位深度", type: "enum", default: "10", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "10", label: "10-bit", recommended: true }, { value: "8", label: "8-bit" }] },
        { id: "chroma_subsampling", label: "色度采样", type: "enum", default: "444", required: false, advanced: true, allow_override: true, supports_expression: false,
          options: [{ value: "444", label: "4:4:4", recommended: true }, { value: "422", label: "4:2:2" }, { value: "420", label: "4:2:0" }] },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.tiff_encoder": {
    plugin_id: "photopipeline.plugins.tiff_encoder", name: "TIFF 编码器", version: "1.0.0", category: "Export",
    description: "编码为 TIFF 格式，支持 LZW/ZIP 压缩", sections: [
      { id: "encoding", label: "编码", collapsible: false, default_collapsed: false, fields: [
        { id: "compression", label: "压缩", type: "enum", default: "lzw", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "none", label: "无" }, { value: "lzw", label: "LZW", recommended: true }, { value: "zip", label: "ZIP" }] },
        { id: "bit_depth", label: "位深度", type: "enum", default: "16", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "16", label: "16-bit", recommended: true }, { value: "8", label: "8-bit" }] },
      ]},
      { id: "metadata", label: "元数据", collapsible: false, default_collapsed: false, fields: [
        { id: "embed_icc", label: "嵌入 ICC", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "big_tiff", label: "BigTIFF", description: "输出 >4GB 时启用", type: "boolean", default: false, required: false, advanced: false, allow_override: true, supports_expression: false },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },

  "photopipeline.plugins.png_encoder": {
    plugin_id: "photopipeline.plugins.png_encoder", name: "PNG 编码器", version: "1.0.0", category: "Export",
    description: "编码为 PNG 格式，支持 16-bit", sections: [
      { id: "encoding", label: "编码", collapsible: false, default_collapsed: false, fields: [
        { id: "compression_level", label: "压缩级别", type: "integer", default: 6, min: 0, max: 9, step: 1, required: true, advanced: false, allow_override: true, supports_expression: false },
        { id: "filter_strategy", label: "滤镜策略", type: "enum", default: "adaptive", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "none", label: "无" }, { value: "sub", label: "Sub" }, { value: "adaptive", label: "自适应", recommended: true }] },
        { id: "bit_depth", label: "位深度", type: "enum", default: "16", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "16", label: "16-bit", recommended: true }, { value: "8", label: "8-bit" }] },
      ]},
      { id: "metadata", label: "元数据", collapsible: false, default_collapsed: false, fields: [
        { id: "embed_icc", label: "嵌入 ICC", type: "boolean", default: true, required: false, advanced: false, allow_override: true, supports_expression: false },
        { id: "color_type", label: "颜色类型", type: "enum", default: "rgb", required: true, advanced: false, allow_override: true, supports_expression: false,
          options: [{ value: "rgb", label: "RGB", recommended: true }, { value: "rgba", label: "RGBA" }, { value: "grayscale", label: "灰度" }] },
      ]},
    ], aux_views: [], preview: "none", min_panel_width: 320,
  },
};
