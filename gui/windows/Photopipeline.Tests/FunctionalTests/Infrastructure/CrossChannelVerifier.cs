using System.Linq;
using SkiaSharp;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public static class CrossChannelVerifier
{
    public static void VerifyEquivalence(string apiOutput, string uiOutput, string testName)
    {
        if (!File.Exists(apiOutput))
            throw new FileNotFoundException($"API output not found: {apiOutput}");

        if (!File.Exists(uiOutput))
            throw new FileNotFoundException($"UI output not found: {uiOutput}");

        var failures = new List<string>();

        // 1. Pixel-level comparison
        try
        {
            ImageAssert.PixelsEqual(apiOutput, uiOutput, tolerancePerChannel: 0);
        }
        catch (Xunit.Sdk.XunitException ex)
        {
            failures.Add($"Pixels: {ex.Message}");
        }

        // 2. Metadata comparison (EXIF must match between API and UI channels)
        try
        {
            var apiMeta = ImageAssert.ReadImageMetadata(apiOutput);
            var uiMeta = ImageAssert.ReadImageMetadata(uiOutput);

            if (apiMeta.Make != uiMeta.Make)
                failures.Add($"Metadata Make mismatch: API='{apiMeta.Make}', UI='{uiMeta.Make}'");
            if (apiMeta.Model != uiMeta.Model)
                failures.Add($"Metadata Model mismatch: API='{apiMeta.Model}', UI='{uiMeta.Model}'");
            if (apiMeta.LensModel != uiMeta.LensModel)
                failures.Add($"Metadata LensModel mismatch: API='{apiMeta.LensModel}', UI='{uiMeta.LensModel}'");

            // DateTime comparison (both absent or matching)
            bool apiHasDate = !string.IsNullOrEmpty(apiMeta.DateTimeOriginal);
            bool uiHasDate = !string.IsNullOrEmpty(uiMeta.DateTimeOriginal);
            if (apiHasDate != uiHasDate)
                failures.Add($"Metadata DateTimeOriginal presence mismatch: API has date={apiHasDate}, UI has date={uiHasDate}");
            else if (apiHasDate && apiMeta.DateTimeOriginal != uiMeta.DateTimeOriginal)
                failures.Add($"Metadata DateTimeOriginal mismatch: API='{apiMeta.DateTimeOriginal}', UI='{uiMeta.DateTimeOriginal}'");

            // ISO comparison
            if (apiMeta.Iso != uiMeta.Iso)
                failures.Add($"Metadata ISO mismatch: API={apiMeta.Iso}, UI={uiMeta.Iso}");
        }
        catch (Exception ex)
        {
            failures.Add($"Metadata read error: {ex.Message}");
        }

        // 3. Format comparison
        var apiExt = Path.GetExtension(apiOutput).TrimStart('.').ToUpperInvariant();
        var uiExt = Path.GetExtension(uiOutput).TrimStart('.').ToUpperInvariant();
        if (!apiExt.Equals(uiExt, StringComparison.OrdinalIgnoreCase))
            failures.Add($"Format mismatch: API={apiExt}, UI={uiExt}");

        // 4. Bit depth comparison
        try
        {
            using var apiBmp = ImageAssert.LoadBitmap(apiOutput);
            using var uiBmp = ImageAssert.LoadBitmap(uiOutput);
            if (apiBmp.BytesPerPixel != uiBmp.BytesPerPixel)
                failures.Add($"Bit depth mismatch: API={apiBmp.BytesPerPixel * 8} bpp, UI={uiBmp.BytesPerPixel * 8} bpp");

            // Size comparison
            if (apiBmp.Width != uiBmp.Width || apiBmp.Height != uiBmp.Height)
                failures.Add($"Size mismatch: API={apiBmp.Width}x{apiBmp.Height}, UI={uiBmp.Width}x{uiBmp.Height}");
        }
        catch (Exception ex)
        {
            failures.Add($"Bitmap analysis error: {ex.Message}");
        }

        // 5. File size comparison (warn if drastically different)
        try
        {
            var apiSize = new FileInfo(apiOutput).Length;
            var uiSize = new FileInfo(uiOutput).Length;
            double ratio = apiSize > 0 ? (double)Math.Max(apiSize, uiSize) / Math.Min(apiSize, uiSize) : 1;
            if (ratio > 2.0)
                failures.Add($"File size ratio {ratio:F2}x exceeds threshold: API={apiSize}B, UI={uiSize}B");
        }
        catch { /* best-effort */ }

        // Aggregate failures
        if (failures.Count > 0)
        {
            var diffDir = Path.Combine(Path.GetTempPath(), "photopipeline_tests", "diffs");
            Directory.CreateDirectory(diffDir);
            var diffPath = Path.Combine(diffDir, $"{Sanitize(testName)}_diff.png");
            try { SaveDiffImage(apiOutput, uiOutput, diffPath); } catch { /* best-effort */ }

            var errorMsg = $"Cross-channel verification FAILED for '{testName}':\n" +
                           string.Join("\n", failures.Select((f, i) => $"  [{i + 1}] {f}")) +
                           $"\nDiff image saved to: {diffPath}\n" +
                           $"API output: {apiOutput}\n" +
                           $"UI output: {uiOutput}";
            throw new Xunit.Sdk.XunitException(errorMsg);
        }
    }

    private static void SaveDiffImage(string pathA, string pathB, string diffPath)
    {
        using var bmpA = ImageAssert.LoadBitmap(pathA);
        using var bmpB = ImageAssert.LoadBitmap(pathB);

        int w = Math.Min(bmpA.Width, bmpB.Width);
        int h = Math.Min(bmpA.Height, bmpB.Height);

        var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul);
        using var diffBmp = new SKBitmap(info);

        for (int y = 0; y < h; y++)
        {
            for (int x = 0; x < w; x++)
            {
                var ca = bmpA.GetPixel(x, y);
                var cb = bmpB.GetPixel(x, y);
                byte dr = (byte)Math.Min(Math.Abs(ca.Red - cb.Red) * 10, 255);
                byte dg = (byte)Math.Min(Math.Abs(ca.Green - cb.Green) * 10, 255);
                byte db = (byte)Math.Min(Math.Abs(ca.Blue - cb.Blue) * 10, 255);
                diffBmp.SetPixel(x, y, new SKColor(dr, dg, db));
            }
        }

        using var image = SKImage.FromBitmap(diffBmp);
        using var data = image.Encode(SKEncodedImageFormat.Png, 100);
        using var stream = File.OpenWrite(diffPath);
        data.SaveTo(stream);
    }

    private static string Sanitize(string name)
    {
        foreach (var c in Path.GetInvalidFileNameChars())
            name = name.Replace(c, '_');
        return name;
    }
}
