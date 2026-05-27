using MetadataExtractor;
using MetadataExtractor.Formats.Exif;
using MetadataExtractor.Formats.Xmp;
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
        byte firstActualR = 0, firstActualG = 0, firstActualB = 0, firstActualA = 0;
        byte firstExpectedR = 0, firstExpectedG = 0, firstExpectedB = 0, firstExpectedA = 0;

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
                        firstActualR = ap.Red; firstActualG = ap.Green; firstActualB = ap.Blue; firstActualA = ap.Alpha;
                        firstExpectedR = ep.Red; firstExpectedG = ep.Green; firstExpectedB = ep.Blue; firstExpectedA = ep.Alpha;
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
                $"actual=({firstActualR},{firstActualG},{firstActualB},{firstActualA}) " +
                $"expected=({firstExpectedR},{firstExpectedG},{firstExpectedB},{firstExpectedA})");
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
        double maxVal = GetMaxValueForColorType(actualBmp.ColorType);
        double psnr = mse < 1e-10 ? double.PositiveInfinity : 10.0 * Math.Log10(maxVal * maxVal / mse);

        if (psnr < minPSNR_dB)
            throw new Xunit.Sdk.XunitException(
                $"PSNR too low: {psnr:F2}dB < minimum {minPSNR_dB:F2}dB");
    }

    public static void SSIMAbove(string actualPath, string referencePath, double minSSIM, bool useWindowed = false)
    {
        using var actualBmp = LoadBitmap(actualPath);
        using var refBmp = LoadBitmap(referencePath);

        if (actualBmp.Width != refBmp.Width || actualBmp.Height != refBmp.Height)
            throw new Xunit.Sdk.XunitException(
                $"Size mismatch: actual={actualBmp.Width}x{actualBmp.Height}, expected={refBmp.Width}x{refBmp.Height}");

        double ssim = useWindowed ? ComputeWindowedSSIM(actualBmp, refBmp) : ComputeSSIM(actualBmp, refBmp);
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

        // Verify file magic bytes match the extension (defense against format forgery)
        VerifyFileHeader(path, ext);

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
        if (!File.Exists(path))
            throw new FileNotFoundException($"Image not found: {path}");

        var metadata = ReadImageMetadata(path);
        assertions(metadata);
    }

    /// <summary>
    /// Reads EXIF and XMP metadata from an image file using MetadataExtractor.
    /// Populates an <see cref="ImageMetadata"/> object with camera, exposure, and GPS data.
    /// </summary>
    public static ImageMetadata ReadImageMetadata(string path)
    {
        var directories = ImageMetadataReader.ReadMetadata(path);
        var metadata = new ImageMetadata();

        // ── EXIF IFD0 (camera maker / model) ──
        var ifd0 = directories.OfType<ExifIfd0Directory>().FirstOrDefault();
        if (ifd0 != null)
        {
            metadata.Make = ifd0.GetDescription(ExifDirectoryBase.TagMake);
            metadata.Model = ifd0.GetDescription(ExifDirectoryBase.TagModel);
        }

        // ── EXIF SubIFD (exposure, ISO, focal length, lens) ──
        var subIfd = directories.OfType<ExifSubIfdDirectory>().FirstOrDefault();
        if (subIfd != null)
        {
            metadata.DateTimeOriginal = subIfd.GetDescription(ExifDirectoryBase.TagDateTimeOriginal);
            metadata.ExposureTime = subIfd.GetDescription(ExifDirectoryBase.TagExposureTime);
            metadata.FNumber = subIfd.GetDescription(ExifDirectoryBase.TagFNumber);

            if (subIfd.TryGetInt32(ExifDirectoryBase.TagIsoEquivalent, out int iso))
                metadata.Iso = (uint)iso;

            metadata.FocalLength = subIfd.GetDescription(ExifDirectoryBase.TagFocalLength);
            metadata.LensModel = subIfd.GetDescription(ExifDirectoryBase.TagLensModel);
        }

        // ── GPS ──
        var gps = directories.OfType<GpsDirectory>().FirstOrDefault();
        if (gps != null)
        {
            var geoLocation = gps.GetGeoLocation();
            if (geoLocation != null)
            {
                metadata.Latitude = geoLocation.Latitude;
                metadata.Longitude = geoLocation.Longitude;
            }
        }

        // ── XMP (fallback for LensModel and other fields not in EXIF) ──
        var xmp = directories.OfType<XmpDirectory>().FirstOrDefault();
        if (xmp != null)
        {
            if (string.IsNullOrEmpty(metadata.LensModel))
            {
                var xmpLens = xmp.GetXmpProperties()
                    .FirstOrDefault(p => p.Key.Contains("Lens", StringComparison.OrdinalIgnoreCase));
                if (!string.IsNullOrEmpty(xmpLens.Value))
                    metadata.LensModel = xmpLens.Value;
            }

            if (string.IsNullOrEmpty(metadata.Make))
            {
                var xmpMake = xmp.GetXmpProperties()
                    .FirstOrDefault(p => p.Key.Contains("Make", StringComparison.OrdinalIgnoreCase));
                if (!string.IsNullOrEmpty(xmpMake.Value))
                    metadata.Make = xmpMake.Value;
            }
        }

        return metadata;
    }

    public static void ApiEqualsUi(string apiOutput, string uiOutput)
    {
        // 1. Pixel-level comparison
        PixelsEqual(apiOutput, uiOutput, tolerancePerChannel: 0);

        // 2. Format comparison
        var apiExt = Path.GetExtension(apiOutput).TrimStart('.').ToUpperInvariant();
        var uiExt = Path.GetExtension(uiOutput).TrimStart('.').ToUpperInvariant();
        if (!apiExt.Equals(uiExt, StringComparison.OrdinalIgnoreCase))
            throw new Xunit.Sdk.XunitException(
                $"ApiEqualsUi format mismatch: API={apiExt}, UI={uiExt}");

        // 3. File magic byte verification (defense against format forgery)
        VerifyFileHeader(apiOutput, apiExt);
        VerifyFileHeader(uiOutput, uiExt);

        // 4. Bit depth comparison
        using var apiBmp = LoadBitmap(apiOutput);
        using var uiBmp = LoadBitmap(uiOutput);
        if (apiBmp.BytesPerPixel != uiBmp.BytesPerPixel)
            throw new Xunit.Sdk.XunitException(
                $"ApiEqualsUi bit depth mismatch: API={apiBmp.BytesPerPixel * 8} bpp, UI={uiBmp.BytesPerPixel * 8} bpp");

        // 5. Metadata comparison
        var apiMeta = ReadImageMetadata(apiOutput);
        var uiMeta = ReadImageMetadata(uiOutput);
        if (apiMeta.Make != uiMeta.Make)
            throw new Xunit.Sdk.XunitException(
                $"ApiEqualsUi metadata Make mismatch: API='{apiMeta.Make}', UI='{uiMeta.Make}'");
        if (apiMeta.Model != uiMeta.Model)
            throw new Xunit.Sdk.XunitException(
                $"ApiEqualsUi metadata Model mismatch: API='{apiMeta.Model}', UI='{uiMeta.Model}'");
    }

    // ── Internal helpers ──

    internal static SKBitmap LoadBitmap(string path)
    {
        if (!File.Exists(path))
            throw new FileNotFoundException($"Image not found: {path}");

        using var codec = SKCodec.Create(path, out var result);
        if (codec == null || result != SKCodecResult.Success)
            throw new InvalidOperationException($"Failed to decode image: {path}");

        var colorType = codec.Info.ColorType;
        if (colorType == SKColorType.Unknown)
            colorType = SKColorType.Rgba8888;
        var info = new SKImageInfo(codec.Info.Width, codec.Info.Height, colorType, SKAlphaType.Premul, SrgbColorSpace);
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
        double maxVal = GetMaxValueForColorType(a.ColorType);
        double C1 = (0.01 * maxVal) * (0.01 * maxVal);
        double C2 = (0.03 * maxVal) * (0.03 * maxVal);
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

    private static double ComputeWindowedSSIM(SKBitmap a, SKBitmap b)
    {
        int w = a.Width, h = a.Height;
        if (w < 11 || h < 11)
            return ComputeSSIM(a, b); // fall back to global SSIM for tiny images

        double maxVal = GetMaxValueForColorType(a.ColorType);
        double C1 = (0.01 * maxVal) * (0.01 * maxVal);
        double C2 = (0.03 * maxVal) * (0.03 * maxVal);

        // 11x11 Gaussian window (sigma = 1.5)
        const int winRadius = 5;
        const int winSize = 11;
        double[] gaussian = new double[winSize * winSize];
        double gaussSum = 0;
        for (int dy = -winRadius; dy <= winRadius; dy++)
        {
            for (int dx = -winRadius; dx <= winRadius; dx++)
            {
                double g = Math.Exp(-(dx * dx + dy * dy) / (2.0 * 1.5 * 1.5));
                gaussian[(dy + winRadius) * winSize + (dx + winRadius)] = g;
                gaussSum += g;
            }
        }
        for (int i = 0; i < gaussian.Length; i++)
            gaussian[i] /= gaussSum;

        // Precompute luminance arrays
        double[] lumA = new double[w * h];
        double[] lumB = new double[w * h];
        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                lumA[y * w + x] = Luminance(a.GetPixel(x, y));
                lumB[y * w + x] = Luminance(b.GetPixel(x, y));
            }
        }

        double ssimSum = 0;
        int validWindows = 0;

        for (int y = winRadius; y < h - winRadius; y++)
        {
            for (int x = winRadius; x < w - winRadius; x++)
            {
                double muX = 0, muY = 0;
                for (int dy = -winRadius; dy <= winRadius; dy++)
                {
                    for (int dx = -winRadius; dx <= winRadius; dx++)
                    {
                        double g = gaussian[(dy + winRadius) * winSize + (dx + winRadius)];
                        double lx = lumA[(y + dy) * w + (x + dx)];
                        double ly = lumB[(y + dy) * w + (x + dx)];
                        muX += g * lx;
                        muY += g * ly;
                    }
                }

                double sigmaX2 = 0, sigmaY2 = 0, sigmaXY = 0;
                for (int dy = -winRadius; dy <= winRadius; dy++)
                {
                    for (int dx = -winRadius; dx <= winRadius; dx++)
                    {
                        double g = gaussian[(dy + winRadius) * winSize + (dx + winRadius)];
                        double lx = lumA[(y + dy) * w + (x + dx)] - muX;
                        double ly = lumB[(y + dy) * w + (x + dx)] - muY;
                        sigmaX2 += g * lx * lx;
                        sigmaY2 += g * ly * ly;
                        sigmaXY += g * lx * ly;
                    }
                }

                double ssim = ((2 * muX * muY + C1) * (2 * sigmaXY + C2)) /
                             ((muX * muX + muY * muY + C1) * (sigmaX2 + sigmaY2 + C2));
                ssimSum += ssim;
                validWindows++;
            }
        }

        return validWindows > 0 ? ssimSum / validWindows : 0;
    }

    private static double ComputeHistogramCorrelation(SKBitmap a, SKBitmap b)
    {
        const int bins = 128;
        int w = a.Width, h = a.Height;

        // Per-channel histograms: [R, G, B] x bins
        double[][] histA = { new double[bins], new double[bins], new double[bins] };
        double[][] histB = { new double[bins], new double[bins], new double[bins] };

        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                var pa = a.GetPixel(x, y);
                var pb = b.GetPixel(x, y);
                int[] channelsA = { pa.Red, pa.Green, pa.Blue };
                int[] channelsB = { pb.Red, pb.Green, pb.Blue };
                for (int c = 0; c < 3; c++)
                {
                    int idxA = Math.Clamp(channelsA[c] * bins / 256, 0, bins - 1);
                    int idxB = Math.Clamp(channelsB[c] * bins / 256, 0, bins - 1);
                    histA[c][idxA]++;
                    histB[c][idxB]++;
                }
            }
        }

        // Compute per-channel Pearson correlation and average
        double totalCorr = 0;
        for (int c = 0; c < 3; c++)
        {
            double meanA = histA[c].Sum() / bins;
            double meanB = histB[c].Sum() / bins;

            double num = 0, denA = 0, denB = 0;
            for (int i = 0; i < bins; i++)
            {
                double da = histA[c][i] - meanA;
                double db = histB[c][i] - meanB;
                num += da * db;
                denA += da * da;
                denB += db * db;
            }

            double denom = Math.Sqrt(denA * denB);
            if (denom < 1e-10)
            {
                // Both histograms have zero variance (pure color).
                // Compare the actual per-channel mean pixel value to distinguish
                // different pure colors (e.g. pure red vs pure blue).
                double meanPixelA = 0, meanPixelB = 0;
                int channelIdx = c;
                for (int y2 = 0; y2 < h; y2++)
                {
                    for (int x2 = 0; x2 < w; x2++)
                    {
                        var pa2 = a.GetPixel(x2, y2);
                        var pb2 = b.GetPixel(x2, y2);
                        int[] ca2 = { pa2.Red, pa2.Green, pa2.Blue };
                        int[] cb2 = { pb2.Red, pb2.Green, pb2.Blue };
                        meanPixelA += ca2[channelIdx];
                        meanPixelB += cb2[channelIdx];
                    }
                }
                meanPixelA /= (w * h);
                meanPixelB /= (w * h);
                totalCorr += Math.Abs(meanPixelA - meanPixelB) < 0.5 ? 1.0 : 0.0;
            }
            else
            {
                totalCorr += num / denom;
            }
        }
        return totalCorr / 3.0;
    }

    private static double ComputeMaxCIEDE2000(SKBitmap a, SKBitmap b)
    {
        int w = a.Width, h = a.Height;

        // 2D grid sampling for uniform spatial coverage (100x100 max)
        int gridX = Math.Min(w, 100);
        int gridY = Math.Min(h, 100);
        double maxDE = 0;

        for (int gy = 0; gy < gridY; gy++)
        {
            for (int gx = 0; gx < gridX; gx++)
            {
                int x = gx * (w - 1) / Math.Max(gridX - 1, 1);
                int y = gy * (h - 1) / Math.Max(gridY - 1, 1);
                double de = DeltaE2000(a.GetPixel(x, y), b.GetPixel(x, y));
                if (de > maxDE) maxDE = de;
            }
        }
        return maxDE;
    }

    private static double Luminance(SKColor c) => 0.2126 * c.Red + 0.7152 * c.Green + 0.0722 * c.Blue;

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

    private static double GetMaxValueForColorType(SKColorType colorType) => colorType switch
    {
        SKColorType.Rgba16161616 or SKColorType.Rgb16161616 => 65535.0,
        SKColorType.RgbaF16 or SKColorType.RgbaF32 => 1.0,
        SKColorType.Alpha8 or SKColorType.Gray8 => 255.0,
        _ => 255.0
    };

    private static void VerifyFileHeader(string path, string ext)
    {
        var expectedMagic = ext.ToUpperInvariant() switch
        {
            "PNG" => new byte[] { 0x89, 0x50, 0x4E, 0x47 },
            "JPEG" or "JPG" => new byte[] { 0xFF, 0xD8, 0xFF },
            "TIFF" or "TIF" => new byte[] { 0x49, 0x49, 0x2A, 0x00 },
            "WEBP" => new byte[] { 0x52, 0x49, 0x46, 0x46 },
            "BMP" => new byte[] { 0x42, 0x4D },
            _ => null
        };
        if (expectedMagic is null) return; // Unknown format, skip

        var header = new byte[expectedMagic.Length];
        using var fs = new FileStream(path, FileMode.Open, FileAccess.Read);
        int read = fs.Read(header, 0, header.Length);
        if (read < header.Length)
            throw new Xunit.Sdk.XunitException(
                $"File too short for {ext} header: expected {header.Length} bytes, got {read}");

        for (int i = 0; i < header.Length; i++)
        {
            if (header[i] != expectedMagic[i])
                throw new Xunit.Sdk.XunitException(
                    $"Format forgery detected: {path} has .{ext} extension but magic bytes at offset {i} " +
                    $"are 0x{header[i]:X2}, expected 0x{expectedMagic[i]:X2}");
        }
    }
}
