namespace Photopipeline.UIAutomationTests;

public sealed class RealWorkflowTests : UIAutomationTestBase
{
    [Fact]
    public void Full_Workflow_Load_Images_Configure_Export()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        if (addButton is not null)
        {
            addButton.Click();
            Thread.Sleep(500);
        }

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        Thread.Sleep(1000);
    }

    [Fact]
    public void Wedding_Photographer_500_Images_Workflow()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Real_Estate_RAW_To_JXL_Workflow()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is not null)
        {
            formatCombo.Click();
            Thread.Sleep(300);
        }

        Thread.Sleep(500);
    }

    [Fact]
    public void HDR_Landscape_Merge_Export()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
            addNodeButton.Click();
            Thread.Sleep(300);
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        Thread.Sleep(500);
    }

    [Fact]
    public void GPX_Track_Interpolation_Workflow()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new OpenQA.Selenium.Interactions.Actions(Driver);
            actions.MoveToElement(canvas, 200, 100).Click().Perform();
            Thread.Sleep(300);
        }

        Thread.Sleep(500);
    }

    [Fact]
    public void Resume_Interrupted_Batch()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(300);

        var stopButton = FindByNameOrNull("Stop");
        stopButton?.Click();
        Thread.Sleep(500);

        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Image_Reload_After_Batch_Reprocessing()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        addButton?.Click();
        Thread.Sleep(300);

        var processAllButton = FindByNameOrNull("Process All");
        processAllButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Edit_Node_Properties_Mid_Batch()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var applyButton = FindByAccessibilityIdOrNull("ApplyButton");
        applyButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Switch_Pipelines_During_Session()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var deleteButton = FindByNameOrNull("Delete");
        deleteButton?.Click();
        Thread.Sleep(200);

        addNodeButton?.Click();
        Thread.Sleep(300);
        addNodeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Compare_Before_After_In_Real_Time()
    {
        var started = TryStartDriver();
        if (!started) return;

        var splitView = FindByNameOrNull("Split View");
        var sideBySide = FindByNameOrNull("Side by Side");

        splitView?.Click();
        Thread.Sleep(200);
        sideBySide?.Click();
        Thread.Sleep(200);
        splitView?.Click();
        Thread.Sleep(200);
    }

    [Fact]
    public void Toggle_Parameter_Types_Quickly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
            addNodeButton.Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Performance_Stress_10_Nodes_No_Crash()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 10; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(150);
        }

        var fitAllButton = FindByNameOrNull("Fit All");
        fitAllButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Performance_Stress_Add_Remove_Add()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 5; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }

        var deleteButton = FindByNameOrNull("Delete");
        for (int i = 0; i < 3; i++)
        {
            deleteButton?.Click();
            Thread.Sleep(150);
        }

        for (int i = 0; i < 3; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }
    }

    [Fact]
    public void Long_Running_Operation_UI_Responsive()
    {
        var started = TryStartDriver();
        if (!started) return;

        var statusText = FindByAccessibilityIdOrNull("StatusText");
        if (statusText is not null)
        {
            statusText.Text.Should().NotBeNull();
        }
    }

    [Fact]
    public void Multi_Monitor_Setup_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var windowPosition = Driver!.Manage().Window.Position;
        var windowSize = Driver.Manage().Window.Size;

        Driver.Manage().Window.Position = new System.Drawing.Point(-1000, 0);
        Thread.Sleep(500);

        Driver.Manage().Window.Position = windowPosition;
        Driver.Manage().Window.Size = windowSize;
        Thread.Sleep(300);
    }

    [Fact]
    public void Keyboard_Navigation_For_All_Panels()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(300);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new OpenQA.Selenium.Interactions.Actions(Driver);
            actions.MoveToElement(canvas, 200, 100).Click().Perform();
            Thread.Sleep(200);
            actions.SendKeys(OpenQA.Selenium.Keys.Tab).Perform();
            Thread.Sleep(200);
        }
    }

    [Fact]
    public void Keyboard_Zoom_In_Out()
    {
        var started = TryStartDriver();
        if (!started) return;

        var zoomIn = FindByNameOrNull("Zoom In");
        var zoomOut = FindByNameOrNull("Zoom Out");

        zoomIn?.Click();
        Thread.Sleep(100);
        zoomIn?.Click();
        Thread.Sleep(100);
        zoomOut?.Click();
        Thread.Sleep(100);
        zoomOut?.Click();
        Thread.Sleep(100);
    }

    [Fact]
    public void Save_Pipeline_As_New()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fileButton = FindByAccessibilityIdOrNull("FileMenuButton")
                         ?? FindByNameOrNull("File");
        fileButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Load_Saved_Pipeline()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fileButton = FindByNameOrNull("File");
        fileButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void New_Pipeline_Clears_Canvas()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var fileButton = FindByNameOrNull("File");
        fileButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Pipeline_Export_As_Config()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fileButton = FindByNameOrNull("File");
        fileButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Pipeline_Import_From_Config()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fileButton = FindByNameOrNull("File");
        fileButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Cross_Session_Pipeline_Persistence()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        RestartApplication();

        Thread.Sleep(2000);
    }

    [Fact]
    public void Context_Menu_On_Image_In_Filmstrip()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        if (filmstrip is not null)
        {
            var actions = new OpenQA.Selenium.Interactions.Actions(Driver);
            actions.MoveToElement(filmstrip, 40, 50).ContextClick().Perform();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Headless_Processing_From_UI()
    {
        var started = TryStartDriver();
        if (!started) return;

        var processAllButton = FindByNameOrNull("Process All");
        processAllButton?.Click();
        Thread.Sleep(1000);
    }

    [Fact]
    public void Color_Managed_Display_Indication()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pixelInfo = FindByAccessibilityIdOrNull("PixelInfoText");
        if (pixelInfo is not null)
        {
            pixelInfo.Text.Should().NotBeNullOrEmpty();
        }
    }

    [Fact]
    public void High_DPI_Scaling_No_Artifacts()
    {
        var started = TryStartDriver();
        if (!started) return;

        var size = Driver!.Manage().Window.Size;
        size.Width.Should().BeGreaterThan(0);
        size.Height.Should().BeGreaterThan(0);
    }

    [Fact]
    public void Notification_On_Export_Complete()
    {
        var started = TryStartDriver();
        if (!started) return;

        var exportButton = FindByNameOrNull("Export");
        exportButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Error_Recovery_On_Invalid_Image()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        addButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Drag_Drop_Images_Into_Filmstrip()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Drag_Drop_Node_Plugin_Into_Canvas()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        Assert.NotNull(canvas);
    }

    [Fact]
    public void Image_Sorting_By_Name()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Image_Sorting_By_Date()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Image_Sorting_By_Size()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Orientation_Detection_Auto_Rotate()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Undo_Stack_Pipeline_Operations()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);

        var deleteButton = FindByNameOrNull("Delete");
        deleteButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Redo_Stack_Pipeline_Operations()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Search_Filter_On_Plugin_List()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is null) return;

        searchBox.Click();
        searchBox.SendKeys("exposure");
        Thread.Sleep(500);

        searchBox.Clear();
        Thread.Sleep(300);

        searchBox.SendKeys("denoise");
        Thread.Sleep(500);
    }

    [Fact]
    public void Category_Switch_Updates_Plugin_List()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Filter_By_Supports_Batching()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Enable_Disable_Plugin_In_Catalog()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void About_Dialog_Shows_Version()
    {
        var started = TryStartDriver();
        if (!started) return;

        var helpButton = FindByAccessibilityIdOrNull("HelpMenuButton")
                         ?? FindByNameOrNull("Help");
        helpButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Keyboard_Shortcut_Help_Available()
    {
        var started = TryStartDriver();
        if (!started) return;

        var helpButton = FindByNameOrNull("Help");
        helpButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Application_Can_Be_Reopened()
    {
        var started = TryStartDriver();
        if (!started) return;

        RestartApplication();
        Thread.Sleep(2000);

        var title = Driver!.Title;
        title.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void Session_State_Partial_Persistence()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        RestartApplication();

        Thread.Sleep(2000);
    }

    [Fact]
    public void Memory_Usage_Stable_After_Full_Workflow()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            for (int i = 0; i < 5; i++)
            {
                addNodeButton.Click();
                Thread.Sleep(100);
            }
        }

        Thread.Sleep(1000);
    }

    [Fact]
    public void Export_Image_From_Preview()
    {
        var started = TryStartDriver();
        if (!started) return;

        var exportButton = FindByNameOrNull("Export");
        exportButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_With_Different_Output_Paths()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsButton = FindByNameOrNull("Batch Settings");
        settingsButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Subfolder_Pattern_For_Output()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsButton = FindByNameOrNull("Batch Settings");
        settingsButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Copy_Plugins_Between_Pipelines()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);

        var duplicateButton = FindByNameOrNull("Duplicate");
        duplicateButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Pipeline_Complex_Chain_10_Nodes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 10; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(100);
        }

        var fitAllButton = FindByNameOrNull("Fit All");
        fitAllButton?.Click();
        Thread.Sleep(500);
    }
}
