using System.Security.Cryptography;
using SkiaSharp;
using Xunit;

namespace Photopipeline.Tests.TestInfrastructure;

public static class ImageAssert
{
    // ---- Pixel-exact comparison ----
    public static void ArePixelEqual(string expectedPath, string actualPath, int tolerancePerChannel = 0)
    {
        Assert.True(File.Exists(expectedPath), $"Expected image not found: {expectedPath}");
        Assert.True(File.Exists(actualPath), $"Actual image not found: {actualPath}");

        using var a = SKBitmap.Decode(expectedPath);
        using var b = SKBitmap.Decode(actualPath);

        Assert.NotNull(a);
        Assert.NotNull(b);
        Assert.Equal(a.Width, b.Width);
        Assert.Equal(a.Height, b.Height);

        for (int y = 0; y < a.Height; y++)
        {
            for (int x = 0; x < a.Width; x++)
            {
                var ca = a.GetPixel(x, y);
                var cb = b.GetPixel(x, y);
                int dr = Math.Abs(ca.Red - cb.Red);
                int dg = Math.Abs(ca.Green - cb.Green);
                int db = Math.Abs(ca.Blue - cb.Blue);
                int da = Math.Abs(ca.Alpha - cb.Alpha);

                bool ok = dr <= tolerancePerChannel && dg <= tolerancePerChannel &&
                          db <= tolerancePerChannel && da <= tolerancePerChannel;
                Assert.True(ok,
                    $"Pixel mismatch at ({x},{y}): expected ({ca.Red},{ca.Green},{ca.Blue},{ca.Alpha}) " +
                    $"actual ({cb.Red},{cb.Green},{cb.Blue},{cb.Alpha})");
            }
        }
    }

    // ---- PSNR ----
    public static double ComputePSNR(string aPath, string bPath)
    {
        using var a = SKBitmap.Decode(aPath) ?? throw new FileNotFoundException("Cannot decode " + aPath);
        using var b = SKBitmap.Decode(bPath) ?? throw new FileNotFoundException("Cannot decode " + bPath);

        if (a.Width != b.Width || a.Height != b.Height)
            return double.NegativeInfinity;

        double mse = 0;
        int count = a.Width * a.Height * 3; // RGB channels

        for (int y = 0; y < a.Height; y++)
        {
            for (int x = 0; x < a.Width; x++)
            {
                var ca = a.GetPixel(x, y);
                var cb = b.GetPixel(x, y);
                mse += (ca.Red - cb.Red) * (ca.Red - cb.Red);
                mse += (ca.Green - cb.Green) * (ca.Green - cb.Green);
                mse += (ca.Blue - cb.Blue) * (ca.Blue - cb.Blue);
            }
        }

        mse /= count;
        if (mse < 1e-10) return double.PositiveInfinity;
        return 20.0 * Math.Log10(255.0 / Math.Sqrt(mse));
    }

    public static void HavePSNRAbove(string aPath, string bPath, double minPSNRdB)
    {
        double psnr = ComputePSNR(aPath, bPath);
        Assert.True(psnr >= minPSNRdB,
            $"PSNR {psnr:F2} dB below threshold {minPSNRdB} dB. File A: {aPath}, File B: {bPath}");
    }

    // ---- Dimensions ----
    public static void HaveDimensions(string path, int expectedW, int expectedH)
    {
        using var bmp = SKBitmap.Decode(path);
        Assert.NotNull(bmp);
        Assert.Equal(expectedW, bmp.Width);
        Assert.Equal(expectedH, bmp.Height);
    }

    // ---- Hash ----
    public static string ComputePixelHash(string path)
    {
        using var bmp = SKBitmap.Decode(path) ?? throw new FileNotFoundException("Cannot decode " + path);
        using var ms = new MemoryStream();
        for (int y = 0; y < bmp.Height; y++)
        {
            for (int x = 0; x < bmp.Width; x++)
            {
                var c = bmp.GetPixel(x, y);
                ms.WriteByte(c.Red);
                ms.WriteByte(c.Green);
                ms.WriteByte(c.Blue);
            }
        }
        ms.Position = 0;
        var hash = SHA256.HashData(ms);
        return Convert.ToHexString(hash).ToLowerInvariant();
    }

    public static void HaveSameHash(string aPath, string bPath)
    {
        var hashA = ComputePixelHash(aPath);
        var hashB = ComputePixelHash(bPath);
        Assert.Equal(hashA, hashB);
    }

    // ---- Statistics ----
    public static void HaveMeanPixelValueInRange(string path, int channel, double min, double max)
    {
        using var bmp = SKBitmap.Decode(path) ?? throw new FileNotFoundException("Cannot decode " + path);
        double sum = 0;
        int count = bmp.Width * bmp.Height;

        for (int y = 0; y < bmp.Height; y++)
        {
            for (int x = 0; x < bmp.Width; x++)
            {
                var c = bmp.GetPixel(x, y);
                sum += channel switch
                {
                    0 => c.Red,
                    1 => c.Green,
                    2 => c.Blue,
                    _ => c.Alpha
                };
            }
        }

        double mean = sum / count;
        Assert.True(mean >= min && mean <= max,
            $"Mean value {mean:F2} of channel {channel} not in range [{min}, {max}]");
    }
}
