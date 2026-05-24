using System.IO.Compression;

namespace Photopipeline.Tests.TestInfrastructure;

/// <summary>
/// Generates test PNG images programmatically using raw PNG encoding.
/// No external image libraries required — writes PNGs directly via DeflateStream.
/// </summary>
public static class TestImageGenerator
{
    // ── PNG constants ─────────────────────────────────────────────
    private static readonly byte[] PngSignature =
        { 137, 80, 78, 71, 13, 10, 26, 10 };

    private const byte ColorTypeRgb = 2;
    private const byte BitDepth8 = 8;

    // ── CRC-32 table (polynomial 0xEDB88320) ─────────────────────
    private static readonly uint[] CrcTable = BuildCrcTable();

    private static uint[] BuildCrcTable()
    {
        var table = new uint[256];
        const uint poly = 0xEDB88320;
        for (uint n = 0; n < 256; n++)
        {
            uint c = n;
            for (int k = 0; k < 8; k++)
            {
                if ((c & 1) != 0)
                    c = poly ^ (c >> 1);
                else
                    c >>= 1;
            }
            table[n] = c;
        }
        return table;
    }

    // ── CRC helpers ────────────────────────────────────────────────
    private static uint ComputeCrc32(byte[] buf, int offset, int len)
    {
        uint crc = 0xFFFFFFFF;
        for (int i = offset; i < offset + len; i++)
            crc = CrcTable[(crc ^ buf[i]) & 0xFF] ^ (crc >> 8);
        return crc ^ 0xFFFFFFFF;
    }

    private static uint ComputeCrc32(byte[] buf)
        => ComputeCrc32(buf, 0, buf.Length);

    // ── Big-endian helpers ────────────────────────────────────────
    private static void WriteBeUInt32(byte[] buf, int offset, uint v)
    {
        buf[offset]     = (byte)(v >> 24);
        buf[offset + 1] = (byte)(v >> 16);
        buf[offset + 2] = (byte)(v >> 8);
        buf[offset + 3] = (byte)v;
    }

    // ── Core PNG writer ───────────────────────────────────────────
    private static void WritePng(string outputPath, int width, int height,
        byte[] rawPixelData)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(outputPath)!);

        using var fs = new FileStream(outputPath, FileMode.Create,
            FileAccess.Write, FileShare.None);

        // -- PNG signature --
        fs.Write(PngSignature, 0, PngSignature.Length);

        // -- IHDR --
        var ihdr = new byte[13];
        WriteBeUInt32(ihdr, 0, (uint)width);
        WriteBeUInt32(ihdr, 4, (uint)height);
        ihdr[8]  = BitDepth8;
        ihdr[9]  = ColorTypeRgb;
        ihdr[10] = 0; // compression
        ihdr[11] = 0; // filter method
        ihdr[12] = 0; // interlace
        WriteChunk(fs, "IHDR", ihdr);

        // -- IDAT (zlib-compressed raw pixel data) --
        byte[] compressed;
        using (var ms = new MemoryStream())
        {
            using (var zlib = new ZLibStream(ms, CompressionLevel.Optimal, true))
                zlib.Write(rawPixelData, 0, rawPixelData.Length);
            compressed = ms.ToArray();
        }
        WriteChunk(fs, "IDAT", compressed);

        // -- IEND --
        WriteChunk(fs, "IEND", Array.Empty<byte>());
    }

    private static void WriteChunk(FileStream fs, string type, byte[] data)
    {
        // Length (4 bytes, big-endian)
        Span<byte> lenBuf = stackalloc byte[4];
        WriteBeUInt32(lenBuf, (uint)data.Length);
        fs.Write(lenBuf);

        // Type (4 bytes ASCII)
        var typeBytes = System.Text.Encoding.ASCII.GetBytes(type);
        fs.Write(typeBytes, 0, 4);

        // Data
        if (data.Length > 0)
            fs.Write(data, 0, data.Length);

        // CRC of type + data
        var crcInput = new byte[4 + data.Length];
        Array.Copy(typeBytes, 0, crcInput, 0, 4);
        if (data.Length > 0)
            Array.Copy(data, 0, crcInput, 4, data.Length);
        uint crc = ComputeCrc32(crcInput);

        Span<byte> crcBuf = stackalloc byte[4];
        WriteBeUInt32(crcBuf, crc);
        fs.Write(crcBuf);
    }

    private static void WriteBeUInt32(Span<byte> buf, uint v)
    {
        buf[0] = (byte)(v >> 24);
        buf[1] = (byte)(v >> 16);
        buf[2] = (byte)(v >> 8);
        buf[3] = (byte)v;
    }

    // ── Raw-pixel-data builders ───────────────────────────────────

    /// <summary>
    /// Builds raw pixel data: each row is [filter_byte(0x00)] + [R,G,B]*width.
    /// </summary>
    private static byte[] BuildRawPixels(int width, int height,
        Func<int, int, (byte r, byte g, byte b)> colorFunc)
    {
        int rowSize = 1 + width * 3;
        var raw = new byte[rowSize * height];

        for (int y = 0; y < height; y++)
        {
            int rowOff = y * rowSize;
            raw[rowOff] = 0; // filter None
            for (int x = 0; x < width; x++)
            {
                int p = rowOff + 1 + x * 3;
                var (r, g, b) = colorFunc(x, y);
                raw[p]     = r;
                raw[p + 1] = g;
                raw[p + 2] = b;
            }
        }
        return raw;
    }

    // ── Public API ─────────────────────────────────────────────────

    /// <summary>
    /// Generates a solid-color PNG file.
    /// </summary>
    public static void GenerateSolidPng(string outputPath, int width,
        int height, byte r, byte g, byte b)
    {
        var raw = BuildRawPixels(width, height, (_, _) => (r, g, b));
        WritePng(outputPath, width, height, raw);
    }

    /// <summary>
    /// Generates a left-to-right gradient PNG (each channel 0→255).
    /// </summary>
    public static void GenerateGradientPng(string outputPath, int width,
        int height)
    {
        var raw = BuildRawPixels(width, height, (x, _) =>
        {
            byte c = (byte)(x * 255.0 / (width - 1));
            return (c, c, c);
        });
        WritePng(outputPath, width, height, raw);
    }

    /// <summary>
    /// Generates a checkerboard-pattern PNG with the given tile size.
    /// </summary>
    public static void GenerateCheckerboardPng(string outputPath, int width,
        int height, int tileSize = 16)
    {
        var raw = BuildRawPixels(width, height, (x, y) =>
        {
            bool white = ((x / tileSize) + (y / tileSize)) % 2 == 0;
            byte v = white ? (byte)255 : (byte)0;
            return (v, v, v);
        });
        WritePng(outputPath, width, height, raw);
    }

    /// <summary>
    /// Generates a color-bars PNG: 8 vertical bars
    /// (white, yellow, cyan, green, magenta, red, blue, black).
    /// </summary>
    public static void GenerateColorBarsPng(string outputPath, int width,
        int height)
    {
        // Pre-defined bar colors
        var colors = new (byte r, byte g, byte b)[]
        {
            (255, 255, 255), // white
            (255, 255, 0),   // yellow
            (0,   255, 255), // cyan
            (0,   255, 0),   // green
            (255, 0,   255), // magenta
            (255, 0,   0),   // red
            (0,   0,   255), // blue
            (0,   0,   0),   // black
        };

        int barWidth = width / 8;
        var raw = BuildRawPixels(width, height, (x, _) =>
        {
            int bar = Math.Min(x / barWidth, 7);
            return colors[bar];
        });
        WritePng(outputPath, width, height, raw);
    }

    /// <summary>
    /// Generates grayscale-step bars (black → white in <paramref name="steps"/> bars).
    /// </summary>
    public static void GenerateGrayscaleStepsPng(string outputPath, int width,
        int height, int steps = 8)
    {
        int barWidth = width / steps;
        var raw = BuildRawPixels(width, height, (x, _) =>
        {
            int step = Math.Min(x / barWidth, steps - 1);
            byte v = (byte)(step * 255.0 / (steps - 1));
            return (v, v, v);
        });
        WritePng(outputPath, width, height, raw);
    }

    /// <summary>
    /// Generates a complete set of test images into the given directory.
    /// Creates: solid_64x64.png, solid_256x256.png, gradient_256x256.png,
    /// gradient_1024x768.png, checkerboard_128x128.png, color_bars_256x128.png,
    /// grayscale_steps_256x16.png, solid_small_32x32.png,
    /// solid_large_1024x1024.png, checkerboard_256x256.png
    /// </summary>
    public static void GenerateTestImageSet(string outputDir)
    {
        Directory.CreateDirectory(outputDir);

        GenerateSolidPng(  Path.Combine(outputDir, "solid_64x64.png"),      64,   64,   128, 128, 128);
        GenerateSolidPng(  Path.Combine(outputDir, "solid_256x256.png"),   256,  256,  200, 100, 50);
        GenerateSolidPng(  Path.Combine(outputDir, "solid_small_32x32.png"), 32,  32,   50,  200, 100);
        GenerateSolidPng(  Path.Combine(outputDir, "solid_large_1024x1024.png"), 1024, 1024, 30, 60, 120);

        GenerateGradientPng(      Path.Combine(outputDir, "gradient_256x256.png"),   256,  256);
        GenerateGradientPng(      Path.Combine(outputDir, "gradient_1024x768.png"), 1024,  768);

        GenerateCheckerboardPng(  Path.Combine(outputDir, "checkerboard_128x128.png"), 128, 128);
        GenerateCheckerboardPng(  Path.Combine(outputDir, "checkerboard_256x256.png"), 256, 256, 32);

        GenerateColorBarsPng(     Path.Combine(outputDir, "color_bars_256x128.png"),    256, 128);

        GenerateGrayscaleStepsPng(Path.Combine(outputDir, "grayscale_steps_256x16.png"), 256, 16);
    }
}
