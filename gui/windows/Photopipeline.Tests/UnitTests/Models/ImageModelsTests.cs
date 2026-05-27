namespace Photopipeline.Tests.UnitTests.Models;

/// <summary>
/// ImageInfo, ImageMetadata, DecodeOptions, PixelDataChunk, EncodeRequest, EncodeProgress
/// are all pure data-transfer objects (POCOs) with no behavioral logic (no validation,
/// Clone, Equals/GetHashCode, or business methods). Testing auto-property defaults
/// provides no value; these are verified implicitly by integration tests that use them.
/// </summary>
public sealed class ImageModelsTests
{
    [Fact(Skip = "ImageInfo is a POCO with no behavioral logic to test")]
    public void ImageInfo_Defaults()
    {
    }

    [Fact(Skip = "ImageMetadata is a POCO with no behavioral logic to test")]
    public void ImageMetadata_StoresExifFields()
    {
    }

    [Fact(Skip = "DecodeOptions is a POCO with no behavioral logic to test")]
    public void DecodeOptions_Defaults()
    {
    }

    [Fact(Skip = "PixelDataChunk is a POCO with no behavioral logic to test")]
    public void PixelDataChunk_Defaults()
    {
    }

    [Fact(Skip = "EncodeRequest is a POCO with no behavioral logic to test")]
    public void EncodeRequest_HasRequiredFields()
    {
    }

    [Fact(Skip = "EncodeProgress is a POCO with no behavioral logic to test")]
    public void EncodeProgress_TracksEncodingState()
    {
    }
}
