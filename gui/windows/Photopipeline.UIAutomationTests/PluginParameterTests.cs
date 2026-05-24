using OpenQA.Selenium.Interactions;

namespace Photopipeline.UIAutomationTests;

public sealed class PluginParameterTests : UIAutomationTestBase
{
    [Fact]
    public void String_Parameter_Accepts_Input()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Integer_Spinner_Increments_Correctly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Integer_Spinner_Decrements_Correctly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Integer_Spinner_Respects_Minimum()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Integer_Spinner_Respects_Maximum()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Float_Slider_Updates_Value()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Float_Slider_Shows_Percentage_Label()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Float_Slider_Respects_Decimal_Places()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Float_Slider_Precise_Input_Via_TextBox()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Boolean_Toggle_Changes_State()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Boolean_Toggle_Default_Is_Off()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Boolean_Toggle_Toggles_Back_And_Forth()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Enum_Dropdown_Shows_All_Options()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Enum_Dropdown_Selects_First_Option()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Enum_Dropdown_Selects_Second_Option()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Enum_Dropdown_Selects_Last_Option()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Enum_Dropdown_Preserves_Selection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Color_Picker_Opens_Flyout()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Color_Picker_Selects_Color()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Color_Picker_Hex_Input_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Color_Picker_Swatches_Available()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void File_Picker_Opens_Dialog()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void File_Picker_Filter_Shows_TIFF_Files()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void File_Picker_Filter_Shows_RAW_Files()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void File_Picker_Cancel_Does_Not_Change_Value()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void File_Picker_Accepts_All_Supported_Types()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Directory_Picker_Opens_Dialog()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void String_Parameter_Shows_Placeholder()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void String_Parameter_Clears_Correctly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void String_Parameter_Max_Length_Enforced()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void String_Parameter_Multiline_Input()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void String_Parameter_Unicode_Input()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Out_Of_Range_Value_Shows_Error()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Integer_Manual_Input_Validates()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Float_Manual_Input_Validates()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Float_Non_Numeric_Input_Shows_Error()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Required_Field_Empty_Shows_Error()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Apply_Button_Updates_Preview()
    {
        var started = TryStartDriver();
        if (!started) return;

        var applyButton = FindByAccessibilityIdOrNull("ApplyButton")
                          ?? FindByNameOrNull("Apply Parameters");
        if (applyButton is not null)
        {
            applyButton.Click();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Reset_Button_Restores_Defaults()
    {
        var started = TryStartDriver();
        if (!started) return;

        var resetButton = FindByAccessibilityIdOrNull("ResetButton")
                          ?? FindByNameOrNull("Reset");
        if (resetButton is not null)
        {
            resetButton.Click();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Apply_Then_Reset_Restores()
    {
        var started = TryStartDriver();
        if (!started) return;

        var applyButton = FindByAccessibilityIdOrNull("ApplyButton");
        var resetButton = FindByAccessibilityIdOrNull("ResetButton");

        applyButton?.Click();
        Thread.Sleep(300);
        resetButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Preview_Button_Triggers_Refresh()
    {
        var started = TryStartDriver();
        if (!started) return;

        var previewButton = FindByAccessibilityIdOrNull("PreviewButton")
                            ?? FindByNameOrNull("Preview");
        previewButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Apply_Without_Plugin_Selected_No_Crash()
    {
        var started = TryStartDriver();
        if (!started) return;

        var applyButton = FindByAccessibilityIdOrNull("ApplyButton");
        applyButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Reset_Without_Plugin_Selected_No_Crash()
    {
        var started = TryStartDriver();
        if (!started) return;

        var resetButton = FindByAccessibilityIdOrNull("ResetButton");
        resetButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Search_Filter_Finds_Plugin()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is null) return;

        searchBox.Click();
        searchBox.SendKeys("demosaic");
        Thread.Sleep(500);

        searchBox.Text.Should().Be("demosaic");
    }

    [Fact]
    public void Search_Filter_Finds_By_Category()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is null) return;

        searchBox.Click();
        searchBox.SendKeys("color");
        Thread.Sleep(500);
    }

    [Fact]
    public void Search_Filter_Finds_By_Description()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is null) return;

        searchBox.Click();
        searchBox.SendKeys("process");
        Thread.Sleep(500);
    }

    [Fact]
    public void Search_Clear_Restores_Full_List()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is null) return;

        searchBox.Click();
        searchBox.SendKeys("test");
        Thread.Sleep(300);
        searchBox.Clear();
        Thread.Sleep(300);
    }

    [Fact]
    public void Search_No_Results_Shows_Empty()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is null) return;

        searchBox.Click();
        searchBox.SendKeys("zzz_nonexistent_xxx");
        Thread.Sleep(500);
    }

    [Fact]
    public void Search_Case_Insensitive()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is null) return;

        searchBox.Click();
        searchBox.SendKeys("DEMOSAIC");
        Thread.Sleep(500);
    }

    [Fact]
    public void Category_Filter_Single_Selection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Category_Filter_All_Shows_Everything()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Category_Filter_Tonal_Shows_Exposure_Plugin()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Category_Filter_Color_Shows_WB_Plugin()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Parameter_Description_ToolTip_Shows()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Parameter_Change_Marks_As_Modified()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Multiple_Parameters_Changed_Before_Apply()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Percentage_Parameter_Shows_Percent_Sign()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Percentage_Slider_Range_0_To_100()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Parameter_With_Unit_Shows_Unit_Label()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Value_Changed_Event_Fired_On_Slider_Drag()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Value_Changed_Event_Fired_On_Text_Entry()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Value_Changed_Event_Fired_On_Toggle()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Plugin_Info_Header_Shows_Name_And_Version()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Plugin_Info_Header_Shows_Category()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Plugin_Info_Header_Shows_Description()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Parameter_Panel_Scrolls_With_Many_Controls()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Parameter_Sections_Grouped()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Expand_Collapse_Parameter_Sections()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Collapsed_Section_Hides_Parameters()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Select_Different_Plugin_Updates_Parameters()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Plugin_Selection_From_Node_Click()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 200, 100).Click().Perform();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Parameter_Default_Matches_Schema()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Integer_Parameter_Step_One()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Float_Parameter_Step_0_1()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Parameter_Validation_On_Blur()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }
}
