using SkiaSharp;

namespace Photopipeline.Tests.FunctionalTests.Infrastructure;

public sealed class GenerateTestDataHelper
{
    // Run manually to regenerate test data:
    //   dotnet test Photopipeline.sln --filter "FullyQualifiedName~GenerateTestData"
    [Fact(Skip = "Run manually to regenerate test data images")]
    public void GenerateTestData()
    {
        var inputDir = Path.Combine(
            AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..",
            "FunctionalTests", "TestData", "input");

        TestImageGenerator.GenerateAll(inputDir);

        Assert.True(Directory.Exists(inputDir));
        Assert.True(File.Exists(Path.Combine(inputDir, "manifest.json")));
    }
}
