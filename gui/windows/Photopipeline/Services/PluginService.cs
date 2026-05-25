using Photopipeline.Models;

namespace Photopipeline.Services;

public sealed class PluginService : IPluginService
{
    private readonly List<PluginInfo> _plugins;
    private readonly Dictionary<string, List<PluginInfo>> _categoryIndex;

    public PluginService()
    {
        _plugins = CreatePluginCatalog();
        _categoryIndex = _plugins.GroupBy(p => p.Category)
            .ToDictionary(g => g.Key, g => g.ToList());
    }

    public Task<IReadOnlyList<PluginInfo>> GetAllAsync(CancellationToken ct = default)
        => Task.FromResult<IReadOnlyList<PluginInfo>>(_plugins);

    public Task<NodeSchema?> GetSchemaAsync(string pluginId, CancellationToken ct = default)
    {
        var plugin = _plugins.FirstOrDefault(p => p.Id == pluginId);
        if (plugin is null) return Task.FromResult<NodeSchema?>(null);

        return Task.FromResult<NodeSchema?>(new NodeSchema
        {
            PluginId = plugin.Id,
            Name = plugin.Name,
            Version = plugin.Version,
            Category = plugin.Category,
            Description = plugin.Description,
            ParameterSchema = plugin.ParameterSchema,
            GuiSchema = new Dictionary<string, object>()
        });
    }

    public IReadOnlyList<string> GetCategories()
        => _categoryIndex.Keys.OrderBy(k => k).ToList();

    public IReadOnlyList<PluginInfo> Search(string query)
        => _plugins.Where(p =>
            p.Name.Contains(query, StringComparison.OrdinalIgnoreCase) ||
            p.Description.Contains(query, StringComparison.OrdinalIgnoreCase) ||
            p.Category.Contains(query, StringComparison.OrdinalIgnoreCase))
            .ToList();

    public IReadOnlyList<PluginInfo> FilterByCategory(string category)
        => _categoryIndex.TryGetValue(category, out var list)
            ? list
            : new List<PluginInfo>();

    private static List<PluginInfo> CreatePluginCatalog() => new()
    {
        // ── Input ──
        new()
        {
            Id = "raw_decoder", Name = "Raw Decoder", Version = "1.0.0", Category = "Input",
            Description = "Decode camera raw files (DNG, NEF, CR2, ARW, ORF) into linear RGB pixel data",
            ParameterSchema = new()
            {
                ["demosaic"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "AMaZE", ["values"] = new[] { "AMaZE", "LMMSE", "VNG4", "PPG", "Bilinear" }, ["description"] = "Demosaicing algorithm" },
                ["border"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 3, ["min"] = 0, ["max"] = 8, ["description"] = "Pixels to crop from border" },
                ["white_balance"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "Camera", ["values"] = new[] { "Camera", "AsShot", "Daylight", "Tungsten", "Fluorescent", "Auto", "Custom" }, ["description"] = "White balance mode" }
            }
        },
        new()
        {
            Id = "file_scanner", Name = "File Scanner", Version = "1.0.0", Category = "Input",
            Description = "Scan directories for supported image files with recursive option and format filtering",
            ParameterSchema = new()
            {
                ["directory"] = new Dictionary<string, object> { ["type"] = "path", ["default"] = "", ["description"] = "Source directory path" },
                ["pattern"] = new Dictionary<string, object> { ["type"] = "string", ["default"] = "*.*", ["description"] = "File pattern glob" },
                ["recursive"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Scan subdirectories recursively" }
            }
        },

        // ── Metadata ──
        new()
        {
            Id = "exif_reader", Name = "EXIF Reader", Version = "1.0.0", Category = "Metadata",
            Description = "Extract EXIF, XMP, and ICC metadata from image files",
            ParameterSchema = new()
            {
                ["extract_exif"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Extract EXIF metadata" },
                ["extract_xmp"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Extract XMP metadata" },
                ["extract_icc"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Extract ICC profile" }
            }
        },
        new()
        {
            Id = "exif_writer", Name = "EXIF Writer", Version = "1.0.0", Category = "Metadata",
            Description = "Embed EXIF, XMP, copyright, and GPS metadata into output images",
            ParameterSchema = new()
            {
                ["preserve_source"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Preserve source metadata" },
                ["copyright"] = new Dictionary<string, object> { ["type"] = "string", ["default"] = "", ["description"] = "Copyright string" },
                ["artist"] = new Dictionary<string, object> { ["type"] = "string", ["default"] = "", ["description"] = "Artist/Creator name" },
                ["embed_icc"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Embed ICC profile" }
            }
        },

        // ── Color ──
        new()
        {
            Id = "white_balance", Name = "White Balance", Version = "1.0.0", Category = "Color",
            Description = "Adjust white balance temperature and tint for color correction",
            ParameterSchema = new()
            {
                ["temperature"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 5500, ["min"] = 2000, ["max"] = 50000, ["unit"] = "K", ["description"] = "Color temperature in Kelvin" },
                ["tint"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.0, ["min"] = -150.0, ["max"] = 150.0, ["step"] = 0.1, ["description"] = "Green/magenta tint" }
            }
        },
        new()
        {
            Id = "color_space", Name = "Color Space Transform", Version = "1.0.0", Category = "Color",
            Description = "Convert between color spaces and apply gamma/transfer functions",
            ParameterSchema = new()
            {
                ["source"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "sRGB", ["values"] = new[] { "sRGB", "AdobeRGB", "ProPhoto", "Display P3", "Rec.2020", "Rec.709", "ACEScct", "XYZ" }, ["description"] = "Source color space" },
                ["target"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "sRGB", ["values"] = new[] { "sRGB", "AdobeRGB", "ProPhoto", "Display P3", "Rec.2020", "Rec.709", "ACEScct" }, ["description"] = "Target color space" },
                ["intent"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "Relative", ["values"] = new[] { "Perceptual", "Relative", "Saturation", "Absolute" }, ["description"] = "Rendering intent" }
            }
        },
        new()
        {
            Id = "lut3d", Name = "3D LUT", Version = "1.0.0", Category = "Color",
            Description = "Apply 3D LUT (cube/3dl) color grading for cinematic look",
            ParameterSchema = new()
            {
                ["lut_path"] = new Dictionary<string, object> { ["type"] = "path", ["default"] = "", ["description"] = "Path to .cube or .3dl LUT file" },
                ["intensity"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 1.0, ["min"] = 0.0, ["max"] = 1.0, ["step"] = 0.01, ["description"] = "LUT blend intensity" }
            }
        },

        // ── Transform ──
        new()
        {
            Id = "resize", Name = "Resize", Version = "1.0.0", Category = "Transform",
            Description = "Resize image with configurable filter and aspect ratio handling",
            ParameterSchema = new()
            {
                ["width"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 1920, ["min"] = 1, ["max"] = 65535, ["description"] = "Target width in pixels" },
                ["height"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 1080, ["min"] = 1, ["max"] = 65535, ["description"] = "Target height in pixels" },
                ["filter"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "Lanczos3", ["values"] = new[] { "Nearest", "Bilinear", "CatmullRom", "Lanczos3", "Mitchell", "Gaussian" }, ["description"] = "Resampling filter" },
                ["keep_aspect"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Preserve aspect ratio" }
            }
        },
        new()
        {
            Id = "crop", Name = "Crop", Version = "1.0.0", Category = "Transform",
            Description = "Crop image to specified region with aspect ratio constraints",
            ParameterSchema = new()
            {
                ["x"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 0, ["min"] = 0, ["description"] = "Left crop coordinate" },
                ["y"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 0, ["min"] = 0, ["description"] = "Top crop coordinate" },
                ["width"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 1920, ["min"] = 1, ["description"] = "Crop width" },
                ["height"] = new Dictionary<string, object> { ["type"] = "int", ["default"] = 1080, ["min"] = 1, ["description"] = "Crop height" },
                ["ratio"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "Free", ["values"] = new[] { "Free", "1:1", "4:3", "3:2", "16:9", "16:10", "5:4" }, ["description"] = "Constrained aspect ratio" }
            }
        },
        new()
        {
            Id = "rotate", Name = "Rotate", Version = "1.0.0", Category = "Transform",
            Description = "Rotate and flip image with auto-orientation support",
            ParameterSchema = new()
            {
                ["angle"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.0, ["min"] = -360.0, ["max"] = 360.0, ["step"] = 0.1, ["unit"] = "deg", ["description"] = "Rotation angle in degrees" },
                ["flip_h"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = false, ["description"] = "Flip horizontally" },
                ["flip_v"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = false, ["description"] = "Flip vertically" },
                ["auto_orient"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Auto-orient from EXIF" }
            }
        },

        // ── Enhance ──
        new()
        {
            Id = "exposure", Name = "Exposure", Version = "1.0.0", Category = "Enhance",
            Description = "Adjust exposure in stops with highlight recovery and contrast",
            ParameterSchema = new()
            {
                ["ev"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.0, ["min"] = -5.0, ["max"] = 5.0, ["step"] = 0.01, ["unit"] = "EV", ["description"] = "Exposure adjustment in stops" },
                ["contrast"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.0, ["min"] = -1.0, ["max"] = 1.0, ["step"] = 0.01, ["description"] = "Contrast adjustment" },
                ["highlight_recovery"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Attempt highlight recovery" }
            }
        },
        new()
        {
            Id = "denoise", Name = "Denoise", Version = "1.0.0", Category = "Enhance",
            Description = "AI-based noise reduction for luminance and chrominance noise",
            ParameterSchema = new()
            {
                ["strength"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.5, ["min"] = 0.0, ["max"] = 1.0, ["step"] = 0.01, ["description"] = "Denoising strength" },
                ["luma"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.5, ["min"] = 0.0, ["max"] = 1.0, ["step"] = 0.01, ["description"] = "Luminance noise reduction" },
                ["chroma"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.5, ["min"] = 0.0, ["max"] = 1.0, ["step"] = 0.01, ["description"] = "Chrominance noise reduction" }
            }
        },
        new()
        {
            Id = "sharpen", Name = "Sharpen", Version = "1.0.0", Category = "Enhance",
            Description = "Image sharpening via unsharp mask with radius and threshold control",
            ParameterSchema = new()
            {
                ["amount"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.5, ["min"] = 0.0, ["max"] = 1.0, ["step"] = 0.01, ["description"] = "Sharpening amount" },
                ["radius"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 1.0, ["min"] = 0.3, ["max"] = 5.0, ["step"] = 0.1, ["unit"] = "px", ["description"] = "Sharpening radius" },
                ["threshold"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 0.0, ["min"] = 0.0, ["max"] = 0.5, ["step"] = 0.001, ["description"] = "Edge threshold to avoid sharpening noise" }
            }
        },

        // ── Format ──
        new()
        {
            Id = "format_convert", Name = "Format Converter", Version = "1.0.0", Category = "Format",
            Description = "Convert between image formats with codec-specific quality settings",
            ParameterSchema = new()
            {
                ["format"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "TIFF", ["values"] = new[] { "TIFF", "JPEG", "PNG", "WebP", "HEIF", "AVIF", "OpenEXR", "JPEG XL", "PPM", "BMP" }, ["description"] = "Output format" },
                ["quality"] = new Dictionary<string, object> { ["type"] = "float", ["default"] = 95.0, ["min"] = 0.0, ["max"] = 100.0, ["description"] = "Quality (JPEG/WebP/HEIF/AVIF)" },
                ["lossless"] = new Dictionary<string, object> { ["type"] = "bool", ["default"] = true, ["description"] = "Use lossless compression when available" },
                ["bit_depth"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "8", ["values"] = new[] { "8", "16", "32" }, ["description"] = "Output bit depth" },
                ["chroma"] = new Dictionary<string, object> { ["type"] = "enum", ["default"] = "4:4:4", ["values"] = new[] { "4:4:4", "4:2:2", "4:2:0" }, ["description"] = "Chroma subsampling" }
            }
        }
    };
}
