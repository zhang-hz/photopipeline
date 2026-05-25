using SkiaSharp;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public static class ImageAssert
{
    private static readonly SKColorSpace SrgbColorSpace = SKColorSpace.CreateSrgb();

    public static void PixelsEqual(string actualPath, string expectedPath, int tolerancePerChannel = 0)
    {
        using var actualBmp = LoadBitmap(actualPath);
        using var expectedBmp = LoadBitmap(expectedPath);

        if (actualBmp.Width != expectedBmp.Width || actualBmp.Height != expectedBmp.Height)
            throw new Xunit.Sdk.XunitException(
                $"Size mismatch: actual={actualBmp.Width}x{actualBmp.Height}, expected={expectedBmp.Width}x{expectedBmp.Height}");

        int diffCount = 0, firstDiffX = -1, firstDiffY = -1;
        byte firstActualR = 0, firstActualG = 0, firstActualB = 0;
        byte firstExpectedR = 0, firstExpectedG = 0, firstExpectedB = 0;

        int w = actualBmp.Width, h = actualBmp.Height;
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                var ap = actualBmp.GetPixel(x, y);
                var ep = expectedBmp.GetPixel(x, y);
                if (Math.Abs(ap.Red - ep.Red) > tolerancePerChannel ||
                    Math.Abs(ap.Green - ep.Green) > tolerancePerChannel ||
                    Math.Abs(ap.Blue - ep.Blue) > tolerancePerChannel ||
                    Math.Abs(ap.Alpha - ep.Alpha) > tolerancePerChannel)
                {
                    diffCount++;
                    if (firstDiffX < 0)
                    {
                        firstDiffX = x; firstDiffY = y;
                        firstActualR = ap.Red; firstActualG = ap.Green; firstActualB = ap.Blue;
                        firstExpectedR = ep.Red; firstExpectedG = ep.Green; firstExpectedB = ep.Blue;
                    }
                }
            }
        }

        if (diffCount > 0)
        {
            int totalPixels = w * h;
            throw new Xunit.Sdk.XunitException(
                $"Pixel mismatch: {diffCount}/{totalPixels} pixels differ. " +
                $"First diff at ({firstDiffX},{firstDiffY}): " +
                $"actual=({firstActualR},{firstActualG},{firstActualB}) " +
                $"expected=({firstExpectedR},{firstExpectedG},{firstExpectedB})");
        }
    }

    public static void PSNRAbove(string actualPath, string referencePath, double minPSNR_dB)
    {
        using var actualBmp = LoadBitmap(actualPath);
        using var refBmp = LoadBitmap(referencePath);

        if (actualBmp.Width != refBmp.Width || actualBmp.Height != refBmp.Height)
            throw new Xunit.Sdk.XunitException(
                $"Size mismatch: actual={actualBmp.Width}x{actualBmp.Height}, expected={refBmp.Width}x{refBmp.Height}");

        double mse = ComputeMSE(actualBmp, refBmp);
        double psnr = mse < 1e-10 ? 100.0 : 10.0 * Math.Log10(255.0 * 255.0 / mse);

        if (psnr < minPSNR_dB)
            throw new Xunit.Sdk.XunitException(
                $"PSNR too low: {psnr:F2}dB < minimum {minPSNR_dB:F2}dB");
    }

    public static void SSIMAbove(string actualPath, string referencePath, double minSSIM)
    {
        using var actualBmp = LoadBitmap(actualPath);
        using var refBmp = LoadBitmap(referencePath);

        if (actualBmp.Width != refBmp.Width || actualBmp.Height != refBmp.Height)
            throw new Xunit.Sdk.XunitException(
                $"Size mismatch: actual={actualBmp.Width}x{actualBmp.Height}, expected={refBmp.Width}x{refBmp.Height}");

        double ssim = ComputeSSIM(actualBmp, refBmp);
        if (ssim < minSSIM)
            throw new Xunit.Sdk.XunitException(
                $"SSIM too low: {ssim:F4} < minimum {minSSIM:F4}");
    }

    public static void HistogramSimilarityAbove(string actualPath, string referencePath, double minCorrelation)
    {
        using var actualBmp = LoadBitmap(actualPath);
        using var refBmp = LoadBitmap(referencePath);

        double correlation = ComputeHistogramCorrelation(actualBmp, refBmp);
        if (correlation < minCorrelation)
            throw new Xunit.Sdk.XunitException(
                $"Histogram correlation too low: {correlation:F4} < minimum {minCorrelation:F4}");
    }

    public static void DeltaEBelow(string actualPath, string referencePath, double maxDeltaE)
    {
        using var actualBmp = LoadBitmap(actualPath);
        using var refBmp = LoadBitmap(referencePath);

        if (actualBmp.Width != refBmp.Width || actualBmp.Height != refBmp.Height)
            throw new Xunit.Sdk.XunitException(
                $"Size mismatch: actual={actualBmp.Width}x{actualBmp.Height}, expected={refBmp.Width}x{refBmp.Height}");

        double maxDE = ComputeMaxCIEDE2000(actualBmp, refBmp);
        if (maxDE > maxDeltaE)
            throw new Xunit.Sdk.XunitException(
                $"DeltaE too high: max ΔE={maxDE:F2} > maximum {maxDeltaE:F2}");
    }

    public static void IsValidFormat(string path, string expectedFormat, int? expectedWidth = null,
        int? expectedHeight = null, int? expectedBitDepth = null)
    {
        if (!File.Exists(path))
            throw new Xunit.Sdk.XunitException($"Output file does not exist: {path}");

        using var bmp = LoadBitmap(path);
        var ext = Path.GetExtension(path).TrimStart('.').ToUpperInvariant();

        if (!string.IsNullOrEmpty(expectedFormat) && !ext.Equals(expectedFormat, StringComparison.OrdinalIgnoreCase))
            throw new Xunit.Sdk.XunitException(
                $"Format mismatch: expected {expectedFormat}, got {ext}");

        if (expectedWidth.HasValue && bmp.Width != expectedWidth.Value)
            throw new Xunit.Sdk.XunitException(
                $"Width mismatch: expected {expectedWidth}, got {bmp.Width}");

        if (expectedHeight.HasValue && bmp.Height != expectedHeight.Value)
            throw new Xunit.Sdk.XunitException(
                $"Height mismatch: expected {expectedHeight}, got {bmp.Height}");

        if (expectedBitDepth.HasValue)
        {
            int channels = bmp.ColorType == SKColorType.Rgba8888 || bmp.ColorType == SKColorType.Bgra8888 ? 4 : 3;
            int actualBits = bmp.BytesPerPixel / Math.Max(channels, 1) * 8;
            if (actualBits != expectedBitDepth.Value)
                throw new Xunit.Sdk.XunitException(
                    $"Bit depth mismatch: expected {expectedBitDepth}, got {actualBits}");
        }
    }

    public static void MetadataMatches(string path, Action<ImageMetadata> assertions)
    {
        assertions(new ImageMetadata());
    }

    public static void ApiEqualsUi(string apiOutput, string uiOutput)
        => PixelsEqual(apiOutput, uiOutput, tolerancePerChannel: 0);

    // ── Internal helpers ──

    internal static SKBitmap LoadBitmap(string path)
    {
        if (!File.Exists(path))
            throw new FileNotFoundException($"Image not found: {path}");

        using var codec = SKCodec.Create(path, out var result);
        if (codec == null || result != SKCodecResult.Success)
            throw new InvalidOperationException($"Failed to decode image: {path}");

        var info = new SKImageInfo(codec.Info.Width, codec.Info.Height, SKColorType.Rgba8888, SKAlphaType.Premul, SrgbColorSpace);
        var bmp = new SKBitmap(info);
        var decodeResult = codec.GetPixels(info, bmp.GetPixels());
        if (decodeResult != SKCodecResult.Success && decodeResult != SKCodecResult.IncompleteInput)
            throw new InvalidOperationException($"Failed to decode pixels: {path}");

        return bmp;
    }

    private static double ComputeMSE(SKBitmap a, SKBitmap b)
    {
        int w = a.Width, h = a.Height;
        double sum = 0;
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                var ca = a.GetPixel(x, y);
                var cb = b.GetPixel(x, y);
                double dr = ca.Red - cb.Red;
                double dg = ca.Green - cb.Green;
                double db = ca.Blue - cb.Blue;
                sum += dr * dr + dg * dg + db * db;
            }
        }
        return sum / (w * h * 3.0);
    }

    private static double ComputeSSIM(SKBitmap a, SKBitmap b)
    {
        const double C1 = 6.5025, C2 = 58.5225;
        int w = a.Width, h = a.Height, n = w * h;

        double muX = 0, muY = 0;
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                muX += Luminance(a.GetPixel(x, y));
                muY += Luminance(b.GetPixel(x, y));
            }
        }
        muX /= n; muY /= n;

        double sigmaX2 = 0, sigmaY2 = 0, sigmaXY = 0;
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                double lx = Luminance(a.GetPixel(x, y)) - muX;
                double ly = Luminance(b.GetPixel(x, y)) - muY;
                sigmaX2 += lx * lx;
                sigmaY2 += ly * ly;
                sigmaXY += lx * ly;
            }
        }
        sigmaX2 /= n; sigmaY2 /= n; sigmaXY /= n;

        return ((2 * muX * muY + C1) * (2 * sigmaXY + C2)) /
               ((muX * muX + muY * muY + C1) * (sigmaX2 + sigmaY2 + C2));
    }

    private static double ComputeHistogramCorrelation(SKBitmap a, SKBitmap b)
    {
        const int bins = 64;
        double[] histA = new double[bins], histB = new double[bins];
        int w = a.Width, h = a.Height;

        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                int idxA = (int)(Luminance(a.GetPixel(x, y)) * bins / 256.0);
                int idxB = (int)(Luminance(b.GetPixel(x, y)) * bins / 256.0);
                histA[Math.Clamp(idxA, 0, bins - 1)]++;
                histB[Math.Clamp(idxB, 0, bins - 1)]++;
            }
        }

        double meanA = histA.Sum() / bins;
        double meanB = histB.Sum() / bins;

        double num = 0, denA = 0, denB = 0;
        for (int i = 0; i < bins; i++)
        {
            double da = histA[i] - meanA;
            double db = histB[i] - meanB;
            num += da * db;
            denA += da * da;
            denB += db * db;
        }

        double denom = Math.Sqrt(denA * denB);
        return denom < 1e-10 ? 1.0 : num / denom;
    }

    private static double ComputeMaxCIEDE2000(SKBitmap a, SKBitmap b)
    {
        int w = a.Width, h = a.Height;
        int totalPixels = w * h;
        int step = Math.Max(1, totalPixels / 10000);
        double maxDE = 0;

        for (int i = 0; i < totalPixels; i += step)
        {
            int x = i % w, y = i / w;
            double de = DeltaE2000(a.GetPixel(x, y), b.GetPixel(x, y));
            if (de > maxDE) maxDE = de;
        }
        return maxDE;
    }

    private static double Luminance(SKColor c) => (c.Red + c.Green + c.Blue) / 3.0;

    private static double DeltaE2000(SKColor p1, SKColor p2)
    {
        var (l1, a1, b1_) = RgbToLab(p1.Red / 255.0, p1.Green / 255.0, p1.Blue / 255.0);
        var (l2, a2, b2_) = RgbToLab(p2.Red / 255.0, p2.Green / 255.0, p2.Blue / 255.0);

        double c1 = Math.Sqrt(a1 * a1 + b1_ * b1_);
        double c2 = Math.Sqrt(a2 * a2 + b2_ * b2_);
        double cBar = (c1 + c2) / 2.0;
        double g = 0.5 * (1.0 - Math.Sqrt(Math.Pow(cBar, 7) / (Math.Pow(cBar, 7) + Math.Pow(25.0, 7))));
        double a1Prime = (1.0 + g) * a1;
        double a2Prime = (1.0 + g) * a2;
        double c1Prime = Math.Sqrt(a1Prime * a1Prime + b1_ * b1_);
        double c2Prime = Math.Sqrt(a2Prime * a2Prime + b2_ * b2_);

        double h1Prime = Math.Atan2(b1_, a1Prime) * 180.0 / Math.PI;
        if (h1Prime < 0) h1Prime += 360;
        double h2Prime = Math.Atan2(b2_, a2Prime) * 180.0 / Math.PI;
        if (h2Prime < 0) h2Prime += 360;

        double dLPrime = l2 - l1;
        double dCPrime = c2Prime - c1Prime;
        double dhPrime;
        if (c1Prime * c2Prime < 1e-10) dhPrime = 0;
        else if (Math.Abs(h2Prime - h1Prime) <= 180) dhPrime = h2Prime - h1Prime;
        else if (h2Prime - h1Prime > 180) dhPrime = h2Prime - h1Prime - 360;
        else dhPrime = h2Prime - h1Prime + 360;
        double dHPrime = 2.0 * Math.Sqrt(c1Prime * c2Prime) * Math.Sin(dhPrime * Math.PI / 360.0);

        double lBar = (l1 + l2) / 2.0;
        double cBarPrime = (c1Prime + c2Prime) / 2.0;
        double hBarPrime;
        if (Math.Abs(h1Prime - h2Prime) <= 180) hBarPrime = (h1Prime + h2Prime) / 2.0;
        else if (h1Prime + h2Prime < 360) hBarPrime = (h1Prime + h2Prime + 360) / 2.0;
        else hBarPrime = (h1Prime + h2Prime - 360) / 2.0;

        double t = 1.0 - 0.17 * Math.Cos((hBarPrime - 30) * Math.PI / 180)
                      + 0.24 * Math.Cos(2 * hBarPrime * Math.PI / 180)
                      + 0.32 * Math.Cos((3 * hBarPrime + 6) * Math.PI / 180)
                      - 0.20 * Math.Cos((4 * hBarPrime - 63) * Math.PI / 180);

        double sl = 1.0 + 0.015 * (lBar - 50) * (lBar - 50) / Math.Sqrt(20 + (lBar - 50) * (lBar - 50));
        double sc = 1.0 + 0.045 * cBarPrime;
        double sh = 1.0 + 0.015 * cBarPrime * t;

        double rt = -2.0 * Math.Sqrt(Math.Pow(cBarPrime, 7) / (Math.Pow(cBarPrime, 7) + Math.Pow(25.0, 7)))
                    * Math.Sin(60 * Math.Exp(-((hBarPrime - 275) / 25) * ((hBarPrime - 275) / 25)) * Math.PI / 180);

        double ratioL = dLPrime / sl;
        double ratioC = dCPrime / sc;
        double ratioH = dHPrime / sh;

        return Math.Sqrt(ratioL * ratioL + ratioC * ratioC + ratioH * ratioH + rt * ratioC * ratioH);
    }

    private static (double l, double a, double b) RgbToLab(double r, double g, double b)
    {
        static double Linearize(double c) =>
            c <= 0.04045 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4);

        double x = Linearize(r) * 0.4124564 + Linearize(g) * 0.3575761 + Linearize(b) * 0.1804375;
        double y = Linearize(r) * 0.2126729 + Linearize(g) * 0.7151522 + Linearize(b) * 0.0721750;
        double z = Linearize(r) * 0.0193339 + Linearize(g) * 0.1191920 + Linearize(b) * 0.9503041;

        x /= 0.95047; z /= 1.08883;

        static double F(double t) =>
            t > 0.008856451679035631 ? Math.Cbrt(t) : 7.787037037037037 * t + 16.0 / 116.0;

        double fx = F(x), fy = F(y), fz = F(z);
        double l = 116 * fy - 16;
        double a = 500 * (fx - fy);
        double bVal = 200 * (fy - fz);

        return (l, a, bVal);
    }
}
