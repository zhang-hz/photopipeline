using SkiaSharp;

namespace Photopipeline.Tests.TestInfrastructure;

public static class TestImageFactory
{
    private static readonly object _lock = new();
    public static string OutputDir
    {
        get
        {
            var dir = Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "..", "TestData");
            dir = Path.GetFullPath(dir);
            Directory.CreateDirectory(dir);
            return dir;
        }
    }

    public static string GetPath(string imageName) => Path.Combine(OutputDir, imageName);

    // ---- Solid color ----
    public static string CreateSolid(string name, int w, int h, SKColor color, SKEncodedImageFormat fmt, int quality = 100)
    {
        var path = GetPath(name);
        if (File.Exists(path)) return path;

        using var bmp = new SKBitmap(w, h);
        bmp.Erase(color);
        Save(bmp, path, fmt, quality);
        return path;
    }

    // ---- Gradient (horizontal, black→white per channel) ----
    public static string CreateGradient(string name, int w, int h, SKEncodedImageFormat fmt)
    {
        var path = GetPath(name);
        if (File.Exists(path)) return path;

        using var bmp = new SKBitmap(w, h);
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                byte v = (byte)(x * 255.0 / (w - 1));
                bmp.SetPixel(x, y, new SKColor(v, v, v));
            }
        }
        Save(bmp, path, fmt);
        return path;
    }

    // ---- Checkerboard ----
    public static string CreateCheckerboard(string name, int w, int h, int tile, SKEncodedImageFormat fmt)
    {
        var path = GetPath(name);
        if (File.Exists(path)) return path;

        using var bmp = new SKBitmap(w, h);
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                bool white = ((x / tile) + (y / tile)) % 2 == 0;
                byte v = white ? (byte)255 : (byte)0;
                bmp.SetPixel(x, y, new SKColor(v, v, v));
            }
        }
        Save(bmp, path, fmt);
        return path;
    }

    // ---- Color bars (8 vertical) ----
    public static string CreateColorBars(string name, int w, int h, SKEncodedImageFormat fmt)
    {
        var path = GetPath(name);
        if (File.Exists(path)) return path;

        var colors = new SKColor[]
        {
            new(255, 255, 255), new(255, 255, 0), new(0, 255, 255), new(0, 255, 0),
            new(255, 0, 255), new(255, 0, 0), new(0, 0, 255), new(0, 0, 0),
        };
        using var bmp = new SKBitmap(w, h);
        int barW = w / 8;
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                int bar = Math.Min(x / barW, 7);
                bmp.SetPixel(x, y, colors[bar]);
            }
        }
        Save(bmp, path, fmt);
        return path;
    }

    // ---- Grayscale steps ----
    public static string CreateGrayscaleSteps(string name, int w, int h, int steps, SKEncodedImageFormat fmt)
    {
        var path = GetPath(name);
        if (File.Exists(path)) return path;

        using var bmp = new SKBitmap(w, h);
        int barW = w / steps;
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                int step = Math.Min(x / barW, steps - 1);
                byte v = (byte)(step * 255.0 / (steps - 1));
                bmp.SetPixel(x, y, new SKColor(v, v, v));
            }
        }
        Save(bmp, path, fmt);
        return path;
    }

    // ---- Noise ----
    public static string CreateNoise(string name, int w, int h, SKEncodedImageFormat fmt)
    {
        var path = GetPath(name);
        if (File.Exists(path)) return path;

        using var bmp = new SKBitmap(w, h);
        var rng = new Random(42);
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                byte v = (byte)rng.Next(256);
                bmp.SetPixel(x, y, new SKColor(v, v, v));
            }
        }
        Save(bmp, path, fmt);
        return path;
    }

    // ---- Full test set ----
    public static void GenerateFullTestSet()
    {
        var marker = Path.Combine(OutputDir, ".factory_generated");
        if (File.Exists(marker)) return;

        lock (_lock)
        {
            if (File.Exists(marker)) return;

        // Small PNG (256x256)
        CreateSolid("solid_rgb_256.png", 256, 256, new SKColor(128, 64, 32), SKEncodedImageFormat.Png);
        CreateGradient("gradient_256.png", 256, 256, SKEncodedImageFormat.Png);
        CreateCheckerboard("checkerboard_256.png", 256, 256, 16, SKEncodedImageFormat.Png);
        CreateColorBars("color_bars_256.png", 256, 128, SKEncodedImageFormat.Png);
        CreateGrayscaleSteps("gray_steps_256.png", 256, 16, 8, SKEncodedImageFormat.Png);
        CreateNoise("noise_256.png", 256, 256, SKEncodedImageFormat.Png);

        // Medium PNG (2048x1536)
        CreateSolid("solid_rgb_2048.png", 2048, 1536, new SKColor(200, 100, 50), SKEncodedImageFormat.Png);
        CreateGradient("gradient_2048.png", 2048, 1536, SKEncodedImageFormat.Png);
        CreateCheckerboard("checkerboard_2048.png", 2048, 1536, 64, SKEncodedImageFormat.Png);

        // Large PNG (4000x3000)
        CreateSolid("solid_rgb_4000.png", 4000, 3000, new SKColor(30, 60, 120), SKEncodedImageFormat.Png);
        CreateGradient("gradient_4000.png", 4000, 3000, SKEncodedImageFormat.Png);

        // JPEG variants
        CreateSolid("solid_rgb_256.jpg", 256, 256, new SKColor(128, 64, 32), SKEncodedImageFormat.Jpeg, 85);
        CreateGradient("gradient_256.jpg", 256, 256, SKEncodedImageFormat.Jpeg);
        CreateColorBars("color_bars_2048.jpg", 2048, 1536, SKEncodedImageFormat.Jpeg);

        // JPEG quality tests
        CreateSolid("jpeg_q100.jpg", 256, 256, new SKColor(100, 150, 200), SKEncodedImageFormat.Jpeg, 100);
        CreateSolid("jpeg_q50.jpg", 256, 256, new SKColor(100, 150, 200), SKEncodedImageFormat.Jpeg, 50);
        CreateSolid("jpeg_q10.jpg", 256, 256, new SKColor(100, 150, 200), SKEncodedImageFormat.Jpeg, 10);

        // WebP
        CreateSolid("solid_rgb_256.webp", 256, 256, new SKColor(128, 64, 32), SKEncodedImageFormat.Webp);
        CreateGradient("gradient_256.webp", 256, 256, SKEncodedImageFormat.Webp);

        // TIFF variants (saved as PNG — SkiaSharp lacks TIFF encoder)
        CreateSolid("solid_rgb_256_tiff.png", 256, 256, new SKColor(128, 64, 32), SKEncodedImageFormat.Png);
        CreateGradient("gradient_256_tiff.png", 256, 256, SKEncodedImageFormat.Png);

        // Additional edge cases
        CreateSolid("solid_1x1.png", 1, 1, new SKColor(255, 0, 0), SKEncodedImageFormat.Png);
        CreateSolid("solid_black_256.png", 256, 256, new SKColor(0, 0, 0), SKEncodedImageFormat.Png);
        CreateSolid("solid_white_256.png", 256, 256, new SKColor(255, 255, 255), SKEncodedImageFormat.Png);
        CreateSolid("solid_red_256.png", 256, 256, new SKColor(255, 0, 0), SKEncodedImageFormat.Png);

        File.WriteAllText(marker, DateTime.Now.ToString("O"));
        }
    }

    // ---- Helpers ----
    private static void Save(SKBitmap bmp, string path, SKEncodedImageFormat fmt, int quality = 100)
    {
        using var img = SKImage.FromBitmap(bmp);
        using var data = img.Encode(fmt, quality);
        using var fs = File.OpenWrite(path);
        data.SaveTo(fs);
    }

    // ---- Queries ----
    public static IEnumerable<string> GetAllPaths() =>
        Directory.Exists(OutputDir) ? Directory.GetFiles(OutputDir) : Enumerable.Empty<string>();

    public static IEnumerable<string> GetByMinSize(int minW, int minH)
    {
        foreach (var path in GetAllPaths())
        {
            using var bmp = SKBitmap.Decode(path);
            if (bmp is not null && bmp.Width >= minW && bmp.Height >= minH)
                yield return path;
        }
    }
}
