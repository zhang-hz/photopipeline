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

        try
        {
            ImageAssert.PixelsEqual(apiOutput, uiOutput, tolerancePerChannel: 0);
        }
        catch (Xunit.Sdk.XunitException ex)
        {
            var diffDir = Path.Combine(Path.GetTempPath(), "photopipeline_tests", "diffs");
            Directory.CreateDirectory(diffDir);
            var diffPath = Path.Combine(diffDir, $"{Sanitize(testName)}_diff.png");
            SaveDiffImage(apiOutput, uiOutput, diffPath);

            throw new Xunit.Sdk.XunitException(
                $"Cross-channel verification FAILED for '{testName}': {ex.Message}\n" +
                $"Diff image saved to: {diffPath}\n" +
                $"API output: {apiOutput}\n" +
                $"UI output: {uiOutput}");
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
