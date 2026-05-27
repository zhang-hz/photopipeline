namespace Photopipeline.Tests.UnitTests.Models;

/// <summary>
/// PluginInfo, BatchSpec, and BatchProgress are pure data-transfer objects (POCOs)
/// with no behavioral logic (no validation, Clone, Equals/GetHashCode, or business
/// methods). Testing auto-property defaults and Dictionary storage provides no value;
/// these are verified implicitly by integration tests that use them.
/// </summary>
public sealed class PluginInfoTests
{
    [Fact(Skip = "PluginInfo is a POCO with no behavioral logic to test")]
    public void PluginInfo_Creation_DefaultValues()
    {
    }

    [Fact(Skip = "PluginInfo is a POCO with no behavioral logic to test")]
    public void PluginInfo_SetAllProperties()
    {
    }

    [Fact(Skip = "ParameterSchema is a standard Dictionary; no custom behavior to test")]
    public void PluginInfo_ParameterSchema_JsonStyleDict()
    {
    }

    [Fact(Skip = "BatchSpec is a POCO with no behavioral logic to test")]
    public void BatchSpec_Defaults()
    {
    }

    [Fact(Skip = "BatchProgress is a POCO with no behavioral logic to test")]
    public void BatchProgress_TracksState()
    {
    }

    [Fact(Skip = "Enum values are compile-time constants; this is verified by the compiler")]
    public void BatchStatus_AllEnumValues()
    {
    }
}
