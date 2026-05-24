namespace Photopipeline.Tests;

/// <summary>
/// One-time test fixture that pre-generates test PNG images for use by UI automation tests.
/// Run with: dotnet test --filter "FullyQualifiedName~TestDataGenerator"
/// </summary>
public sealed class TestDataGenerator
{
    /// <summary>
    /// Generates all 10 test images into the TestData/ directory.
    /// This only runs once; if images already exist it skips regeneration.
    /// </summary>
    [Fact]
    public void Generate_Test_Images()
    {
        TestInfrastructure.TestImageFactory.GenerateFullTestSet();
    }

    [Fact]
    public void Verify_Test_Images_Exist()
    {
        TestInfrastructure.TestImageFactory.GenerateFullTestSet();

        var files = TestInfrastructure.TestImageFactory.GetAllPaths().ToList();
        Assert.True(files.Count >= 25, $"Expected ≥25 test images, found {files.Count}");

        foreach (var file in files)
        {
            var info = new FileInfo(file);
            Assert.True(info.Length > 0, $"Empty test image: {Path.GetFileName(file)}");
        }
    }
}
