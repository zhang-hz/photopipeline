using System.Text.Json;
using SkiaSharp;
using Photopipeline.Models;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public static class TestImageGenerator
{
    private const int Seed = 42;
    private static readonly SKColorSpace SrgbColorSpace = SKColorSpace.CreateSrgb();

    public static void GenerateAll(string outputDir)
    {
        Directory.CreateDirectory(outputDir);
        var manifest = new List<TestImageRecord>();
        var rng = new Random(Seed);

        GenerateSolidImages(outputDir, manifest);
        GenerateGradientImages(outputDir, manifest, rng);
        GenerateCheckerboardImages(outputDir, manifest);
        GenerateColorBarImages(outputDir, manifest);
        GenerateGrayscaleStepImages(outputDir, manifest);
        GenerateZonePlateImages(outputDir, manifest);
        GenerateNaturalTextureImages(outputDir, manifest, rng);
        GenerateAlphaImages(outputDir, manifest, rng);
        GenerateBoundaryImages(outputDir, manifest);
        GenerateFormatVarietyImages(outputDir, manifest);
        GenerateHighBitDepthImages(outputDir, manifest, rng);

        var manifestPath = Path.Combine(outputDir, "manifest.json");
        var json = JsonSerializer.Serialize(manifest, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(manifestPath, json);

        Console.WriteLine($"Generated {manifest.Count} test images in {outputDir}");
    }

    // ── Solid colors ──

    private static void GenerateSolidImages(string dir, List<TestImageRecord> manifest)
    {
        var solids = new[]
        {
            ("pure_black", new SKColor(0, 0, 0)),
            ("pure_white", new SKColor(255, 255, 255)),
            ("pure_red", new SKColor(255, 0, 0)),
            ("pure_green", new SKColor(0, 255, 0)),
            ("pure_blue", new SKColor(0, 0, 255)),
            ("mid_gray_128", new SKColor(128, 128, 128)),
            ("dark_gray_32", new SKColor(32, 32, 32)),
            ("light_gray_224", new SKColor(224, 224, 224)),
            ("pure_cyan", new SKColor(0, 255, 255)),
            ("pure_magenta", new SKColor(255, 0, 255)),
            ("pure_yellow", new SKColor(255, 255, 0)),
            ("navy_blue", new SKColor(16, 32, 128)),
        };

        foreach (var (name, color) in solids)
        {
            // Small version
            var smallPath = Path.Combine(dir, "solid", $"{name}_64x64.png");
            Directory.CreateDirectory(Path.GetDirectoryName(smallPath)!);
            SaveSolid(smallPath, 64, 64, color);
            manifest.Add(CreateRecord($"{name}_small", $"solid/{name}_64x64.png", "solid", 64, 64, 8,
                tags: new[] { "small", "solid", "rgb8" }));

            // Large version
            var largePath = Path.Combine(dir, "solid", $"{name}_1920x1080.png");
            SaveSolid(largePath, 1920, 1080, color);
            manifest.Add(CreateRecord($"{name}_large", $"solid/{name}_1920x1080.png", "solid", 1920, 1080, 8,
                tags: new[] { "large", "solid", "rgb8" }));
        }
    }

    // ── Gradients ──

    private static void GenerateGradientImages(string dir, List<TestImageRecord> manifest, Random rng)
    {
        var specs = new[]
        {
            ("horiz_rgb", GradientDirection.Horizontal, false, 512, 256, 8),
            ("vert_rgb", GradientDirection.Vertical, false, 512, 256, 8),
            ("radial_rgb", GradientDirection.Radial, false, 512, 512, 8),
            ("diag_rgb", GradientDirection.Diagonal, false, 512, 512, 8),
            ("horiz_gray", GradientDirection.Horizontal, true, 256, 128, 8),
            ("vert_gray", GradientDirection.Vertical, true, 256, 128, 8),
            ("radial_gray", GradientDirection.Radial, true, 512, 512, 8),
            ("diag_gray", GradientDirection.Diagonal, true, 512, 512, 8),
            ("horiz_rgb_16bit", GradientDirection.Horizontal, false, 256, 128, 16),
            ("vert_rgb_16bit", GradientDirection.Vertical, false, 256, 128, 16),
        };

        foreach (var (name, dir_, gray, w, h, bd) in specs)
        {
            var path = Path.Combine(dir, "gradient", $"{name}_{w}x{h}.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveGradient(path, w, h, dir_, gray, bd);
            manifest.Add(CreateRecord($"gradient_{name}", $"gradient/{name}_{w}x{h}.png",
                "gradient", w, h, bd,
                tags: new[] { "gradient", gray ? "gray" : "rgb", bd == 8 ? "8bit" : "16bit" }));
        }
    }

    // ── Checkerboard ──

    private static void GenerateCheckerboardImages(string dir, List<TestImageRecord> manifest)
    {
        var specs = new[] { (2, 128), (4, 256), (8, 256), (16, 256), (32, 512), (64, 512) };
        foreach (var (grid, size) in specs)
        {
            var path = Path.Combine(dir, "pattern", $"checkerboard_{grid}x{grid}_{size}.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveCheckerboard(path, size, size, grid, grid);
            manifest.Add(CreateRecord($"checkerboard_{grid}x{grid}", $"pattern/checkerboard_{grid}x{grid}_{size}.png",
                "pattern", size, size, 8,
                tags: new[] { "checkerboard", "pattern", "rgb8", $"grid_{grid}" }));
        }

        // RGBA checkerboard
        var rgbaPath = Path.Combine(dir, "pattern", "checkerboard_rgba_256.png");
        SaveCheckerboardRGBA(rgbaPath, 256, 256, 8, 8);
        manifest.Add(CreateRecord("checkerboard_rgba", "pattern/checkerboard_rgba_256.png",
            "pattern", 256, 256, 8, hasAlpha: true,
            tags: new[] { "checkerboard", "pattern", "rgba" }));
    }

    // ── Color bars ──

    private static void GenerateColorBarImages(string dir, List<TestImageRecord> manifest)
    {
        var specs = new[] { (256, 128, 8), (512, 256, 8), (512, 256, 16) };
        foreach (var (w, h, bd) in specs)
        {
            var path = Path.Combine(dir, "pattern", $"color_bars_{w}x{h}_{bd}bit.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveColorBars(path, w, h, bd);
            manifest.Add(CreateRecord($"color_bars_{bd}bit", $"pattern/color_bars_{w}x{h}_{bd}bit.png",
                "pattern", w, h, bd,
                tags: new[] { "color_bars", "pattern", bd == 8 ? "8bit" : "16bit" }));
        }
    }

    // ── Grayscale steps ──

    private static void GenerateGrayscaleStepImages(string dir, List<TestImageRecord> manifest)
    {
        var specs = new[] { 2, 4, 8, 16, 32, 64, 128, 256 };
        foreach (int steps in specs)
        {
            int w = steps * 4, h = 64;
            var path = Path.Combine(dir, "pattern", $"grayscale_{steps}steps_{w}x{h}.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveGrayscaleSteps(path, w, h, steps);
            manifest.Add(CreateRecord($"grayscale_{steps}steps", $"pattern/grayscale_{steps}steps_{w}x{h}.png",
                "pattern", w, h, 8,
                tags: new[] { "grayscale", "pattern", $"steps_{steps}" }));
        }

        // 16-bit grayscale steps
        var path16 = Path.Combine(dir, "pattern", "grayscale_64steps_16bit_256x64.png");
        SaveGrayscaleSteps16(path16, 256, 64, 64);
        manifest.Add(CreateRecord("grayscale_64steps_16bit", "pattern/grayscale_64steps_16bit_256x64.png",
            "pattern", 256, 64, 16,
            tags: new[] { "grayscale", "pattern", "16bit", "steps_64" }));
    }

    // ── Zone plates ──

    private static void GenerateZonePlateImages(string dir, List<TestImageRecord> manifest)
    {
        var specs = new[] { (128, 128, 4.0), (256, 256, 8.0), (512, 512, 16.0), (1024, 1024, 32.0) };
        foreach (var (w, h, freq) in specs)
        {
            var path = Path.Combine(dir, "zone_plate", $"zone_plate_{freq}hz_{w}x{h}.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveZonePlate(path, w, h, freq);
            manifest.Add(CreateRecord($"zone_plate_{freq}hz", $"zone_plate/zone_plate_{freq}hz_{w}x{h}.png",
                "zone_plate", w, h, 8,
                tags: new[] { "zone_plate", "sine", "pattern", $"freq_{freq}" }));
        }
    }

    // ── Natural texture simulations ──

    private static void GenerateNaturalTextureImages(string dir, List<TestImageRecord> manifest, Random rng)
    {
        var specs = new[] { ("noise_grain", 256, 256), ("noise_clouds", 512, 512),
                            ("noise_wood", 256, 256), ("noise_marble", 512, 512),
                            ("noise_fabric", 256, 256), ("noise_terrain", 512, 256) };

        foreach (var (name, w, h) in specs)
        {
            var path = Path.Combine(dir, "natural", $"{name}_{w}x{h}.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveNaturalTexture(path, w, h, name, rng);
            manifest.Add(CreateRecord(name, $"natural/{name}_{w}x{h}.png",
                "natural", w, h, 8,
                tags: new[] { "natural", "noise", "texture" }));
        }
    }

    // ── Alpha channel images ──

    private sealed record AlphaSpec(string Name, int Width, int Height,
        Func<int, int, int, int, SKColor> PixelFunc);

    private static void GenerateAlphaImages(string dir, List<TestImageRecord> manifest, Random rng)
    {
        var specs = new AlphaSpec[]
        {
            new("alpha_solid_transparent", 128, 128, (x, y, w, h) => new SKColor(255, 0, 0, 128)),
            new("alpha_gradient_transparent", 256, 128, (x, y, w, h) => new SKColor(255, 255, 255, (byte)(y * 255 / h))),
            new("alpha_checker_transparent", 256, 256, (x, y, w, h) =>
            {
                bool cell = (x / 32 % 2 == 0) ^ (y / 32 % 2 == 0);
                return new SKColor(255, 255, 255, cell ? (byte)255 : (byte)64);
            }),
            new("alpha_circle_opaque", 256, 256, (x, y, w, h) =>
            {
                double dx = x - w / 2.0, dy = y - h / 2.0;
                double dist = Math.Sqrt(dx * dx + dy * dy) / (w / 2.0);
                byte alpha = (byte)Math.Clamp((1.0 - dist) * 255, 0, 255);
                return new SKColor(0, 128, 255, alpha);
            }),
            new("alpha_text_overlay", 256, 128, (x, y, w, h) =>
            {
                byte a = (byte)(((x + y) % 64 < 32) ? 255 : 128);
                return new SKColor(255, 255, 255, a);
            }),
            new("alpha_full_opaque", 128, 128, (_, _, _, _) => new SKColor(100, 200, 50, 255)),
        };

        foreach (var spec in specs)
        {
            var path = Path.Combine(dir, "alpha", $"{spec.Name}_{spec.Width}x{spec.Height}.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveAlphaImage(path, spec.Width, spec.Height, spec.PixelFunc);
            manifest.Add(CreateRecord(spec.Name, $"alpha/{spec.Name}_{spec.Width}x{spec.Height}.png",
                "alpha", spec.Width, spec.Height, 8, hasAlpha: true,
                tags: new[] { "alpha", "rgba" }));
        }
    }

    // ── Boundary sizes ──

    private static void GenerateBoundaryImages(string dir, List<TestImageRecord> manifest)
    {
        var specs = new[] { (1, 1), (1, 100), (100, 1), (2, 2), (3, 3), (799, 601), (601, 799) };
        foreach (var (w, h) in specs)
        {
            var path = Path.Combine(dir, "boundary", $"boundary_{w}x{h}.png");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveCheckerboard(path, w, h, 1, 1);
            manifest.Add(CreateRecord($"boundary_{w}x{h}", $"boundary/boundary_{w}x{h}.png",
                "boundary", w, h, 8,
                tags: new[] { "boundary", "edge_case", $"size_{w}x{h}" }));
        }
    }

    // ── Format variety ──

    private static void GenerateFormatVarietyImages(string dir, List<TestImageRecord> manifest)
    {
        var formats = new[] { "PNG", "JPEG", "TIFF", "WEBP", "BMP", "AVIF", "HEIF", "JXL" };
        foreach (var fmt in formats)
        {
            var path = Path.Combine(dir, "format_variety", $"test_256x128.{fmt.ToLowerInvariant()}");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);

            var info = new SKImageInfo(256, 128, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
            using var bmp = new SKBitmap(info);
            using var canvas = new SKCanvas(bmp);
            PaintTestPattern(canvas, 256, 128);
            SaveBitmap(bmp, path, fmt switch
            {
                "JPEG" => SKEncodedImageFormat.Jpeg,
                "TIFF" => SKEncodedImageFormat.Tiff,
                "WEBP" => SKEncodedImageFormat.Webp,
                "BMP" => SKEncodedImageFormat.Bmp,
                // AVIF/HEIF/JXL: saved as PNG (SkiaSharp doesn't encode these natively)
                // but tagged with their format for input-format test awareness
                _ => SKEncodedImageFormat.Png
            },
                fmt switch
                {
                    "JPEG" => 90,
                    "WEBP" => 95,
                    _ => 100
                });

            manifest.Add(CreateRecord($"format_{fmt.ToLowerInvariant()}", $"format_variety/test_256x128.{fmt.ToLowerInvariant()}",
                "format_variety", 256, 128, 8,
                tags: new[] { "format", fmt.ToLowerInvariant() }));
        }
    }

    // ── High bit-depth images ──

    private static void GenerateHighBitDepthImages(string dir, List<TestImageRecord> manifest, Random rng)
    {
        // 16-bit gradient
        var path16 = Path.Combine(dir, "high_bitdepth", "gradient_rgb_16bit_256x128.png");
        Directory.CreateDirectory(Path.GetDirectoryName(path16)!);
        SaveGradient(path16, 256, 128, GradientDirection.Horizontal, false, 16);
        manifest.Add(CreateRecord("gradient_rgb_16bit", "high_bitdepth/gradient_rgb_16bit_256x128.png",
            "high_bitdepth", 256, 128, 16,
            tags: new[] { "16bit", "gradient", "rgb" }));

        // 32-bit float gradient (stored as 16-bit PNG since PNG doesn't support 32-bit float natively)
        // Uses extended-range values (0–4x sRGB) for HDR simulation
        var path32 = Path.Combine(dir, "high_bitdepth", "gradient_rgb_hdr_256x128.png");
        SaveHDRGradient(path32, 256, 128);
        manifest.Add(CreateRecord("gradient_rgb_hdr", "high_bitdepth/gradient_rgb_hdr_256x128.png",
            "high_bitdepth", 256, 128, 16, colorSpace: "LinearRec2020",
            tags: new[] { "hdr", "gradient", "rgb", "float", "high_dynamic_range" }));
    }

    // ── Drawing helpers ──

    private static void SaveSolid(string path, int w, int h, SKColor color)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        using var canvas = new SKCanvas(bmp);
        canvas.Clear(color);
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveGradient(string path, int w, int h, GradientDirection dir_, bool gray, int bitDepth)
    {
        bool is16Bit = bitDepth >= 16;
        var info = new SKImageInfo(w, h,
            is16Bit ? SKColorType.Rgba16161616 : SKColorType.Rgba8888,
            SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);

        if (is16Bit)
        {
            // Write ushort values directly for proper 16-bit precision
            int pixelCount = w * h * 4; // Rgba = 4 channels
            var data = new ushort[pixelCount];
            for (int y = 0; y < h; y++)
            {
                for (int x = 0; x < w; x++)
                {
                    double t;
                    switch (dir_)
                    {
                        case GradientDirection.Horizontal: t = (double)x / w; break;
                        case GradientDirection.Vertical: t = (double)y / h; break;
                        case GradientDirection.Diagonal: t = (x / (double)w + y / (double)h) / 2.0; break;
                        case GradientDirection.Radial:
                            double dx = (x - w / 2.0) / (w / 2.0);
                            double dy = (y - h / 2.0) / (h / 2.0);
                            t = Math.Clamp(Math.Sqrt(dx * dx + dy * dy), 0, 1);
                            break;
                        default: t = 0.5; break;
                    }

                    ushort r, g, b;
                    if (gray)
                    {
                        r = g = b = (ushort)(t * 65535);
                    }
                    else
                    {
                        double hue = t * 6.0;
                        double c = 1.0, xc = c * (1 - Math.Abs(hue % 2 - 1));
                        double r1, g1, b1;
                        switch ((int)hue)
                        {
                            case 0: r1 = c; g1 = xc; b1 = 0; break;
                            case 1: r1 = xc; g1 = c; b1 = 0; break;
                            case 2: r1 = 0; g1 = c; b1 = xc; break;
                            case 3: r1 = 0; g1 = xc; b1 = c; break;
                            case 4: r1 = xc; g1 = 0; b1 = c; break;
                            default: r1 = c; g1 = 0; b1 = xc; break;
                        }
                        r = (ushort)(r1 * 65535); g = (ushort)(g1 * 65535); b = (ushort)(b1 * 65535);
                    }
                    int idx = (y * w + x) * 4;
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = 65535;
                }
            }
            var bytes = new byte[pixelCount * 2];
            System.Buffer.BlockCopy(data, 0, bytes, 0, bytes.Length);
            System.Runtime.InteropServices.Marshal.Copy(bytes, 0, bmp.GetPixels(), bytes.Length);
        }
        else
        {
            using var canvas = new SKCanvas(bmp);
            using var paint = new SKPaint { IsAntialias = false };
            for (int y = 0; y < h; y++)
            {
                for (int x = 0; x < w; x++)
                {
                    double t;
                    switch (dir_)
                    {
                        case GradientDirection.Horizontal: t = (double)x / w; break;
                        case GradientDirection.Vertical: t = (double)y / h; break;
                        case GradientDirection.Diagonal: t = (x / (double)w + y / (double)h) / 2.0; break;
                        case GradientDirection.Radial:
                            double dx = (x - w / 2.0) / (w / 2.0);
                            double dy = (y - h / 2.0) / (h / 2.0);
                            t = Math.Clamp(Math.Sqrt(dx * dx + dy * dy), 0, 1);
                            break;
                        default: t = 0.5; break;
                    }

                    byte r, g, b;
                    if (gray)
                    {
                        r = g = b = (byte)(t * 255);
                    }
                    else
                    {
                        double hue = t * 6.0;
                        double c = 1.0, xc = c * (1 - Math.Abs(hue % 2 - 1));
                        double r1, g1, b1;
                        switch ((int)hue)
                        {
                            case 0: r1 = c; g1 = xc; b1 = 0; break;
                            case 1: r1 = xc; g1 = c; b1 = 0; break;
                            case 2: r1 = 0; g1 = c; b1 = xc; break;
                            case 3: r1 = 0; g1 = xc; b1 = c; break;
                            case 4: r1 = xc; g1 = 0; b1 = c; break;
                            default: r1 = c; g1 = 0; b1 = xc; break;
                        }
                        r = (byte)(r1 * 255); g = (byte)(g1 * 255); b = (byte)(b1 * 255);
                    }
                    paint.Color = new SKColor(r, g, b);
                    canvas.DrawPoint(x, y, paint);
                }
            }
        }
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveCheckerboard(string path, int w, int h, int cols, int rows)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        using var canvas = new SKCanvas(bmp);
        // Cap columns/rows to avoid degenerate or out-of-bounds cells
        int effectiveCols = Math.Min(cols, w);
        int effectiveRows = Math.Min(rows, h);
        int cellW = Math.Max(1, w / effectiveCols);
        int cellH = Math.Max(1, h / effectiveRows);

        using var paintWhite = new SKPaint { Color = SKColors.White };
        using var paintBlack = new SKPaint { Color = SKColors.Black };

        for (int cy = 0; cy < effectiveRows; cy++)
        {
            for (int cx = 0; cx < effectiveCols; cx++)
            {
                var paint = (cx + cy) % 2 == 0 ? paintWhite : paintBlack;
                int x = cx * cellW;
                int y = cy * cellH;
                int drawW = Math.Min(cellW, w - x);
                int drawH = Math.Min(cellH, h - y);
                canvas.DrawRect(x, y, drawW, drawH, paint);
            }
        }
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveCheckerboardRGBA(string path, int w, int h, int cols, int rows)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        using var canvas = new SKCanvas(bmp);
        int cellW = Math.Max(1, w / cols);
        int cellH = Math.Max(1, h / rows);

        using var paintRed = new SKPaint { Color = new SKColor(255, 0, 0, 255) };
        using var paintTransparent = new SKPaint { Color = new SKColor(0, 0, 0, 0) };
        using var paintBlue = new SKPaint { Color = new SKColor(0, 0, 255, 128) };
        using var paintGreen = new SKPaint { Color = new SKColor(0, 255, 0, 192) };

        var paints = new[] { paintRed, paintTransparent, paintBlue, paintGreen };

        for (int cy = 0; cy < rows; cy++)
        {
            for (int cx = 0; cx < cols; cx++)
            {
                var paint = paints[(cx * 3 + cy * 2) % paints.Length];
                canvas.DrawRect(cx * cellW, cy * cellH, cellW, cellH, paint);
            }
        }
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveColorBars(string path, int w, int h, int bitDepth)
    {
        bool is16Bit = bitDepth >= 16;
        var colorType = is16Bit ? SKColorType.Rgba16161616 : SKColorType.Rgba8888;
        var info = new SKImageInfo(w, h, colorType, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);

        // Color values: proper 16-bit (ushort max 65535) or 8-bit (byte max 255)
        var colorValues16 = new (ushort R, ushort G, ushort B)[]
        {
            (65535, 65535, 65535), // White
            (65535, 65535, 0),     // Yellow
            (0, 65535, 65535),     // Cyan
            (0, 65535, 0),         // Green
            (65535, 0, 65535),     // Magenta
            (65535, 0, 0),         // Red
            (0, 0, 65535),         // Blue
            (0, 0, 0),             // Black
        };

        float barW = (float)w / colorValues16.Length;

        if (is16Bit)
        {
            // Write ushort values directly
            int pixelCount = w * h * 4;
            var data = new ushort[pixelCount];
            for (int y = 0; y < h; y++)
            {
                for (int x = 0; x < w; x++)
                {
                    int barIdx = Math.Min((int)(x / barW), colorValues16.Length - 1);
                    var (r, g, b) = colorValues16[barIdx];
                    int idx = (y * w + x) * 4;
                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = 65535;
                }
            }
            var bytes = new byte[pixelCount * 2];
            System.Buffer.BlockCopy(data, 0, bytes, 0, bytes.Length);
            System.Runtime.InteropServices.Marshal.Copy(bytes, 0, bmp.GetPixels(), bytes.Length);
        }
        else
        {
            using var canvas = new SKCanvas(bmp);
            var colors = new[]
            {
                new SKColor(255, 255, 255), // White
                new SKColor(255, 255, 0),   // Yellow
                new SKColor(0, 255, 255),   // Cyan
                new SKColor(0, 255, 0),     // Green
                new SKColor(255, 0, 255),   // Magenta
                new SKColor(255, 0, 0),     // Red
                new SKColor(0, 0, 255),     // Blue
                new SKColor(0, 0, 0),       // Black
            };
            using var paint = new SKPaint();
            for (int i = 0; i < colors.Length; i++)
            {
                paint.Color = colors[i];
                canvas.DrawRect(i * barW, 0, barW, h, paint);
            }
        }
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveGrayscaleSteps(string path, int w, int h, int steps)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        using var canvas = new SKCanvas(bmp);
        float stepW = (float)w / steps;

        using var paint = new SKPaint();
        for (int i = 0; i < steps; i++)
        {
            byte v = (byte)(i * 255 / (steps - 1));
            paint.Color = new SKColor(v, v, v);
            canvas.DrawRect(i * stepW, 0, stepW, h, paint);
        }
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveGrayscaleSteps16(string path, int w, int h, int steps)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba16161616, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);

        // Write ushort values directly for proper 16-bit precision (0..65535)
        int pixelCount = w * h * 4;
        var data = new ushort[pixelCount];
        float stepW = (float)w / steps;

        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                int stepIdx = Math.Min((int)(x / stepW), steps - 1);
                ushort v = (ushort)(stepIdx * 65535 / (steps - 1));
                int idx = (y * w + x) * 4;
                data[idx] = v;
                data[idx + 1] = v;
                data[idx + 2] = v;
                data[idx + 3] = 65535;
            }
        }
        var bytes = new byte[pixelCount * 2];
        System.Buffer.BlockCopy(data, 0, bytes, 0, bytes.Length);
        System.Runtime.InteropServices.Marshal.Copy(bytes, 0, bmp.GetPixels(), bytes.Length);
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveZonePlate(string path, int w, int h, double frequency)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        using var canvas = new SKCanvas(bmp);
        using var paint = new SKPaint { IsAntialias = false };

        double cx = w / 2.0, cy = h / 2.0;
        double maxR = Math.Min(cx, cy);
        double freqScale = frequency / maxR;

        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                double dx = x - cx, dy = y - cy;
                double r = Math.Sqrt(dx * dx + dy * dy);
                double value = 0.5 + 0.5 * Math.Sin(r * r * Math.PI * freqScale / maxR);
                byte v = (byte)(value * 255);
                paint.Color = new SKColor(v, v, v);
                canvas.DrawPoint(x, y, paint);
            }
        }
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveNaturalTexture(string path, int w, int h, string style, Random rng)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        using var canvas = new SKCanvas(bmp);

        var pixels = new byte[w * h * 4];
        for (int i = 0; i < w * h; i++)
        {
            int x = i % w, y = i / w;
            byte r, g, b;
            switch (style)
            {
                case "noise_grain":
                    r = g = b = (byte)rng.Next(256);
                    break;
                case "noise_clouds":
                    double v = SimplexNoise(x * 0.03, y * 0.03, 0, rng);
                    r = g = b = (byte)Math.Clamp((v * 0.5 + 0.5) * 255, 0, 255);
                    break;
                case "noise_wood":
                    double wv = Math.Sin(y * 0.1 + SimplexNoise(x * 0.02, y * 0.02, 0, rng) * 3) * 0.5 + 0.5;
                    r = (byte)(139 + (int)(wv * 40));
                    g = (byte)(90 + (int)(wv * 25));
                    b = (byte)(43 + (int)(wv * 15));
                    break;
                case "noise_marble":
                    double mv = Math.Sin((x + SimplexNoise(x * 0.02, y * 0.02, 1, rng) * 30) * 0.04) * 0.5 + 0.5;
                    r = g = b = (byte)(180 + mv * 75);
                    break;
                case "noise_fabric":
                    double fv = Math.Abs(Math.Sin(x * 0.05) * Math.Cos(y * 0.05)) +
                                SimplexNoise(x * 0.04, y * 0.04, 2, rng) * 0.3;
                    r = (byte)Math.Clamp(fv * 180, 0, 255);
                    g = (byte)Math.Clamp(fv * 200, 0, 255);
                    b = (byte)Math.Clamp(fv * 220, 0, 255);
                    break;
                case "noise_terrain":
                    double tv = SimplexNoise(x * 0.005, y * 0.005, 3, rng);
                    if (tv < 0) { r = 30; g = 60; b = 120; } // water
                    else if (tv < 0.2) { r = 194; g = 178; b = 128; } // sand
                    else if (tv < 0.5) { r = 34; g = 139; b = 34; } // grass
                    else if (tv < 0.7) { r = 100; g = 100; b = 100; } // rock
                    else { r = g = b = 255; } // snow
                    break;
                default:
                    r = g = b = 128;
                    break;
            }
            int idx = i * 4;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }

        var ptr = bmp.GetPixels();
        System.Runtime.InteropServices.Marshal.Copy(pixels, 0, ptr, pixels.Length);
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveAlphaImage(string path, int w, int h, Func<int, int, int, int, SKColor> pixelFunc)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        using var canvas = new SKCanvas(bmp);
        using var paint = new SKPaint { IsAntialias = false };

        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                paint.Color = pixelFunc(x, y, w, h);
                canvas.DrawPoint(x, y, paint);
            }
        }
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void SaveHDRGradient(string path, int w, int h)
    {
        var info = new SKImageInfo(w, h, SKColorType.Rgba16161616, SKAlphaType.Premul, SrgbColorSpace);
        using var bmp = new SKBitmap(info);
        int pixelCount = w * h * 4; // Rgba = 4 channels
        int byteLen = pixelCount * 2;
        var data = new ushort[pixelCount];
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                float t = (float)x / w;
                float val = t * 4.0f;
                ushort v = (ushort)Math.Clamp(val * 65535f / 4.0f, 0, 65535);
                int idx = (y * w + x) * 4;
                data[idx] = v;
                data[idx + 1] = v;
                data[idx + 2] = v;
                data[idx + 3] = 65535;
            }
        }
        var bytes = new byte[byteLen];
        System.Buffer.BlockCopy(data, 0, bytes, 0, byteLen);
        System.Runtime.InteropServices.Marshal.Copy(bytes, 0, bmp.GetPixels(), byteLen);
        SaveBitmap(bmp, path, SKEncodedImageFormat.Png, 100);
    }

    private static void PaintTestPattern(SKCanvas canvas, int w, int h)
    {
        // Color bars on top half
        var colors = new[] { SKColors.White, SKColors.Yellow, SKColors.Cyan,
                             SKColors.Green, SKColors.Magenta, SKColors.Red, SKColors.Blue, SKColors.Black };
        float barW = (float)w / colors.Length;
        using var paint = new SKPaint();
        for (int i = 0; i < colors.Length; i++)
        {
            paint.Color = colors[i];
            canvas.DrawRect(i * barW, 0, barW, h / 2f, paint);
        }
        // Grayscale ramp on bottom half
        for (int x = 0; x < w; x++)
        {
            byte v = (byte)(x * 255 / w);
            paint.Color = new SKColor(v, v, v);
            canvas.DrawRect(x, h / 2f, 1, h / 2f, paint);
        }
    }

    private static void SaveBitmap(SKBitmap bmp, string path, SKEncodedImageFormat format, int quality)
    {
        using var image = SKImage.FromBitmap(bmp);
        using var data = image.Encode(format, quality);
        using var stream = new FileStream(path, FileMode.Create, FileAccess.Write);
        data.SaveTo(stream);
    }

    private static TestImageRecord CreateRecord(string id, string relativePath, string category,
        int width, int height, int bitDepth, bool hasAlpha = false, string[]? tags = null,
        string colorSpace = "sRGB")
        => new()
        {
            Id = id,
            RelativePath = relativePath,
            Category = category,
            Format = Path.GetExtension(relativePath).TrimStart('.').ToUpperInvariant(),
            Width = width,
            Height = height,
            BitDepth = bitDepth,
            ColorSpace = colorSpace,
            HasAlpha = hasAlpha,
            Tags = tags ?? [],
            Description = $"{category} test image {width}x{height} {bitDepth}bit"
        };

    private static double SimplexNoise(double x, double y, int seed, Random rng)
    {
        // Basic value noise (simplified — not true simplex but adequate for test patterns)
        int ix = (int)Math.Floor(x);
        int iy = (int)Math.Floor(y);
        double fx = x - ix;
        double fy = y - iy;

        static double Hash(int x, int y, int seed)
        {
            int h = seed + x * 374761393 + y * 668265263;
            h = (h ^ (h >> 13)) * 1274126177;
            return (h ^ (h >> 16)) / (double)int.MaxValue;
        }

        double v00 = Hash(ix, iy, seed);
        double v10 = Hash(ix + 1, iy, seed);
        double v01 = Hash(ix, iy + 1, seed);
        double v11 = Hash(ix + 1, iy + 1, seed);

        // Smooth interpolation
        fx = fx * fx * (3 - 2 * fx);
        fy = fy * fy * (3 - 2 * fy);

        double a = v00 + (v10 - v00) * fx;
        double b = v01 + (v11 - v01) * fx;
        return a + (b - a) * fy;
    }

    private enum GradientDirection { Horizontal, Vertical, Radial, Diagonal }
}
