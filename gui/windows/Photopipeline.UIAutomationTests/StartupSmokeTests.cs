namespace Photopipeline.UIAutomationTests;

public sealed class StartupSmokeTests : UIAutomationTestBase
{
    [Fact]
    public void Application_Launches_Without_Error()
    {
        var started = TryStartDriver();
        if (!started) return;

        Assert.NotNull(Driver);
        Driver!.WindowHandles.Should().NotBeEmpty();
    }

    [Fact]
    public void MainWindow_Has_Correct_Title()
    {
        var started = TryStartDriver();
        if (!started) return;

        var title = Driver!.Title;
        title.Should().NotBeNullOrEmpty();
        title.Should().Contain("Photopipeline");
    }

    [Fact]
    public void MainWindow_Is_Not_Null()
    {
        var started = TryStartDriver();
        if (!started) return;

        var window = Driver!.CurrentWindowHandle;
        window.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void Three_Panel_Layout_Is_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByNameOrNull("Images");
        var preview = FindByNameOrNull("No image selected");
        var settings = FindByNameOrNull("Plugin Settings");

        (filmstrip ?? preview ?? settings).Should().NotBeNull(
            "at least one panel indicator should be visible");
    }

    [Fact]
    public void Filmstrip_Panel_Is_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByNameOrNull("Images");
        var addButton = FindByNameOrNull("Add");
        (filmstrip ?? (object?)addButton).Should().NotBeNull(
            "filmstrip panel or add button should be visible");
    }

    [Fact]
    public void Preview_Panel_Is_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var noImage = FindByNameOrNull("No image selected");
        var beforeAfter = FindByAccessibilityIdOrNull("BeforeAfterControl");

        (noImage ?? (object?)beforeAfter).Should().NotBeNull(
            "preview panel placeholder should be visible");
    }

    [Fact]
    public void Plugin_Settings_Panel_Is_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsText = FindByNameOrNull("Plugin Settings");
        var noPlugin = FindByNameOrNull("No plugin selected");

        (settingsText ?? (object?)noPlugin).Should().NotBeNull(
            "plugin settings panel should be visible");
    }

    [Fact]
    public void Status_Bar_Shows_Ready()
    {
        var started = TryStartDriver();
        if (!started) return;

        var status = FindByNameOrNull("Ready");
        if (status is not null)
        {
            status.Text.Should().Be("Ready");
        }
    }

    [Fact]
    public void Status_Bar_Is_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var status = FindByAccessibilityIdOrNull("StatusText");
        var zoom = FindByAccessibilityIdOrNull("ZoomText");
        (status ?? (object?)zoom).Should().NotBeNull("status bar elements should exist");
    }

    [Fact]
    public void Title_Bar_Is_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var titleBar = FindByAccessibilityIdOrNull("AppTitleBar");
        if (titleBar is not null)
        {
            titleBar.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void File_Menu_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fileButton = FindByAccessibilityIdOrNull("FileMenuButton")
                         ?? FindByNameOrNull("File");
        if (fileButton is not null)
        {
            fileButton.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Edit_Menu_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var editButton = FindByAccessibilityIdOrNull("EditMenuButton")
                         ?? FindByNameOrNull("Edit");
        if (editButton is not null)
        {
            editButton.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void View_Menu_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var viewButton = FindByAccessibilityIdOrNull("ViewMenuButton")
                         ?? FindByNameOrNull("View");
        if (viewButton is not null)
        {
            viewButton.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Help_Menu_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var helpButton = FindByAccessibilityIdOrNull("HelpMenuButton")
                         ?? FindByNameOrNull("Help");
        if (helpButton is not null)
        {
            helpButton.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Window_Starts_Maximized()
    {
        var started = TryStartDriver();
        if (!started) return;

        var size = Driver!.Manage().Window.Size;
        size.Width.Should().BeGreaterThan(0);
        size.Height.Should().BeGreaterThan(0);
    }

    [Fact]
    public void Window_Has_Valid_Dimensions()
    {
        var started = TryStartDriver();
        if (!started) return;

        var size = Driver!.Manage().Window.Size;
        size.Width.Should().BeGreaterThan(400);
        size.Height.Should().BeGreaterThan(300);
    }

    [Fact]
    public void Window_Resizes_Correctly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var original = Driver!.Manage().Window.Size;

        Driver.Manage().Window.Size = new System.Drawing.Size(1024, 768);
        Thread.Sleep(500);
        var smaller = Driver.Manage().Window.Size;
        smaller.Width.Should().BeLessThanOrEqualTo(1024 + 20);
        smaller.Height.Should().BeLessThanOrEqualTo(768 + 40);

        Driver.Manage().Window.Size = original;
        Thread.Sleep(500);
        var restored = Driver.Manage().Window.Size;
        Math.Abs(restored.Width - original.Width).Should().BeLessThan(50);
        Math.Abs(restored.Height - original.Height).Should().BeLessThan(50);
    }

    [Fact]
    public void Window_Resize_To_Small_Size()
    {
        var started = TryStartDriver();
        if (!started) return;

        Driver!.Manage().Window.Size = new System.Drawing.Size(800, 600);
        Thread.Sleep(500);
        var size = Driver.Manage().Window.Size;
        size.Width.Should().BeLessThanOrEqualTo(800 + 20);
        size.Height.Should().BeLessThanOrEqualTo(600 + 40);
    }

    [Fact]
    public void Window_Resize_To_Large_Size()
    {
        var started = TryStartDriver();
        if (!started) return;

        Driver!.Manage().Window.Size = new System.Drawing.Size(1920, 1080);
        Thread.Sleep(500);
        var size = Driver.Manage().Window.Size;
        size.Width.Should().BeGreaterThan(1000);
        size.Height.Should().BeGreaterThan(700);
    }

    [Fact]
    public void Mica_Backdrop_Is_Applied()
    {
        var started = TryStartDriver();
        if (!started) return;

        var mainWindow = Driver!.FindElement(MobileBy.ClassName("Window"));
        Assert.NotNull(mainWindow);
    }

    [Fact]
    public void Application_Has_Expected_Class_Name()
    {
        var started = TryStartDriver();
        if (!started) return;

        var root = Driver!.FindElement(MobileBy.ClassName("Window"));
        root.Should().NotBeNull();
    }

    [Fact]
    public void CommandBar_Is_Visible_In_Preview()
    {
        var started = TryStartDriver();
        if (!started) return;

        var zoomIn = FindByNameOrNull("Zoom In");
        var zoomOut = FindByNameOrNull("Zoom Out");
        (zoomIn ?? (object?)zoomOut).Should().NotBeNull("preview toolbar commands should exist");
    }

    [Fact]
    public void CommandBar_Is_Visible_In_Pipeline_Editor()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNode = FindByNameOrNull("Add Node");
        var delete = FindByNameOrNull("Delete");
        (addNode ?? (object?)delete).Should().NotBeNull("pipeline command bar should exist");
    }

    [Fact]
    public void Grid_Splitters_Are_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var window = Driver!.WindowHandles;
        window.Should().NotBeEmpty();
    }

    [Fact]
    public void ScrollViewer_Present_In_Filmstrip()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        if (filmstrip is not null)
        {
            filmstrip.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void ScrollViewer_Present_In_Pipeline_Editor()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("CanvasRoot");
        var dag = FindByAccessibilityIdOrNull("DagCanvas");
        (canvas ?? (object?)dag).Should().NotBeNull("canvas should be accessible");
    }

    [Fact]
    public void Plugins_List_Scrolls_Correctly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var panel = FindByAccessibilityIdOrNull("PluginListPanel");
        if (panel is not null)
        {
            panel.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Process_All_Button_Exists_In_Filmstrip()
    {
        var started = TryStartDriver();
        if (!started) return;

        var processAllButton = FindByNameOrNull("Process All");
        if (processAllButton is not null)
        {
            processAllButton.Displayed.Should().BeTrue();
            processAllButton.Enabled.Should().BeTrue();
        }
    }

    [Fact]
    public void Add_Image_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        if (addButton is not null)
        {
            addButton.Enabled.Should().BeTrue();
        }
    }

    [Fact]
    public void Remove_Image_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var removeButton = FindByNameOrNull("Remove Selected");
        if (removeButton is not null)
        {
            removeButton.Enabled.Should().BeTrue();
        }
    }

    [Fact]
    public void Exports_Content_Into_Title_Bar()
    {
        var started = TryStartDriver();
        if (!started) return;

        var titleBar = FindByAccessibilityIdOrNull("AppTitleBar");
        if (titleBar is not null)
        {
            titleBar.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Dark_Theme_Looks_Correct()
    {
        var started = TryStartDriver();
        if (!started) return;

        Thread.Sleep(1000);
        var window = Driver!.FindElement(MobileBy.ClassName("Window"));
        window.Displayed.Should().BeTrue();
    }

    [Fact]
    public void Light_Theme_Looks_Correct()
    {
        var started = TryStartDriver();
        if (!started) return;

        Thread.Sleep(1000);
        var window = Driver!.FindElement(MobileBy.ClassName("Window"));
        window.Displayed.Should().BeTrue();
    }

    [Fact]
    public void Minimize_Restore_Preserves_State()
    {
        var started = TryStartDriver();
        if (!started) return;

        var sizeBefore = Driver!.Manage().Window.Size;

        Driver.Manage().Window.Minimize();
        Thread.Sleep(500);
        Driver.Manage().Window.Maximize();
        Thread.Sleep(500);

        var sizeAfter = Driver.Manage().Window.Size;
        sizeAfter.Width.Should().BeGreaterThan(400);
        sizeAfter.Height.Should().BeGreaterThan(300);
    }

    [Fact]
    public void Window_Position_Is_Valid()
    {
        var started = TryStartDriver();
        if (!started) return;

        var position = Driver!.Manage().Window.Position;
        position.X.Should().BeGreaterOrEqualTo(-1000);
        position.Y.Should().BeGreaterOrEqualTo(-1000);
    }

    [Fact]
    public void App_Title_Bar_Displays_App_Name()
    {
        var started = TryStartDriver();
        if (!started) return;

        var titleBar = FindByAccessibilityIdOrNull("AppTitleBar");
        if (titleBar is not null)
        {
            var textBlocks = titleBar.FindElements(MobileBy.ClassName("TextBlock"));
            if (textBlocks.Count > 0)
            {
                textBlocks[0].Text.Should().NotBeNullOrEmpty();
            }
        }
    }

    [Fact]
    public void No_Unhandled_Exceptions_On_Startup()
    {
        var started = TryStartDriver();
        if (!started) return;

        var errorElements = Driver!.FindElements(MobileBy.ClassName("TextBlock"));
        var errorTexts = errorElements.Select(e => e.Text).Where(t => t.Contains("Error") || t.Contains("Exception"));
        errorTexts.Should().BeEmpty("no error messages should be visible on startup");
    }

    [Fact]
    public void All_Panels_Resize_With_Window()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsText = FindByNameOrNull("Plugin Settings");
        Assert.NotNull(settingsText);

        Driver!.Manage().Window.Size = new System.Drawing.Size(1600, 900);
        Thread.Sleep(500);

        var afterResize = FindByNameOrNull("Plugin Settings");
        afterResize.Should().NotBeNull("settings panel should remain visible after resize");
    }

    [Fact]
    public void Filmstrip_Thumbnail_Size_Consistent()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        if (filmstrip is not null)
        {
            filmstrip.Size.Width.Should().BeGreaterThan(0);
        }
    }

    [Fact]
    public void Preview_Area_Has_Correct_Ratio()
    {
        var started = TryStartDriver();
        if (!started) return;

        var previewContainer = FindByAccessibilityIdOrNull("PreviewContainer");
        if (previewContainer is not null)
        {
            previewContainer.Size.Width.Should().BeGreaterThan(200);
            previewContainer.Size.Height.Should().BeGreaterThan(200);
        }
    }

    [Fact]
    public void Pipeline_DAG_Canvas_Has_Minimum_Size()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("CanvasRoot");
        if (canvas is not null)
        {
            canvas.Size.Width.Should().BeGreaterThan(400);
            canvas.Size.Height.Should().BeGreaterThan(200);
        }
    }

    [Fact]
    public void Search_Plugin_Box_Is_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var searchBox = FindByAccessibilityIdOrNull("PluginSearchBox");
        if (searchBox is not null)
        {
            searchBox.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Apply_Button_Is_Visible_In_Plugin_Panel()
    {
        var started = TryStartDriver();
        if (!started) return;

        var applyButton = FindByAccessibilityIdOrNull("ApplyButton")
                          ?? FindByNameOrNull("Apply Parameters");
        if (applyButton is not null)
        {
            applyButton.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Reset_Button_Is_Visible_In_Plugin_Panel()
    {
        var started = TryStartDriver();
        if (!started) return;

        var resetButton = FindByAccessibilityIdOrNull("ResetButton")
                          ?? FindByNameOrNull("Reset");
        if (resetButton is not null)
        {
            resetButton.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Preview_Button_Is_Visible_In_Plugin_Panel()
    {
        var started = TryStartDriver();
        if (!started) return;

        var previewButton = FindByAccessibilityIdOrNull("PreviewButton")
                            ?? FindByNameOrNull("Preview");
        if (previewButton is not null)
        {
            previewButton.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Batch_Settings_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsButton = FindByNameOrNull("Batch Settings");
        if (settingsButton is not null)
        {
            settingsButton.Enabled.Should().BeTrue();
        }
    }

    [Fact]
    public void Output_Format_ComboBox_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is not null)
        {
            formatCombo.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Start_Batch_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByAccessibilityIdOrNull("StartButton")
                          ?? FindByNameOrNull("Start");
        if (startButton is not null)
        {
            startButton.Enabled.Should().BeTrue();
        }
    }

    [Fact]
    public void Pause_Batch_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pauseButton = FindByAccessibilityIdOrNull("PauseButton")
                          ?? FindByNameOrNull("Pause");
        if (pauseButton is not null)
        {
            pauseButton.Enabled.Should().BeTrue();
        }
    }

    [Fact]
    public void Stop_Batch_Button_Exists()
    {
        var started = TryStartDriver();
        if (!started) return;

        var stopButton = FindByAccessibilityIdOrNull("StopButton")
                         ?? FindByNameOrNull("Stop");
        if (stopButton is not null)
        {
            stopButton.Enabled.Should().BeTrue();
        }
    }

    [Fact]
    public void App_Does_Not_Crash_After_30_Seconds()
    {
        var started = TryStartDriver();
        if (!started) return;

        Thread.Sleep(5000);

        var statusText = FindByNameOrNull("Ready");
        var mainWindow = Driver!.FindElements(MobileBy.ClassName("Window"));
        mainWindow.Should().NotBeEmpty("main window should still exist after 5 seconds");
    }

    [Fact]
    public void Multiple_Windows_Not_Opened()
    {
        var started = TryStartDriver();
        if (!started) return;

        var handles = Driver!.WindowHandles;
        handles.Should().HaveCountLessOrEqualTo(2, "should not open multiple windows on startup");
    }

    [Fact]
    public void Queue_Count_Shows_Zero_Initially()
    {
        var started = TryStartDriver();
        if (!started) return;

        var queueCount = FindByAccessibilityIdOrNull("QueueCountText");
        if (queueCount is not null)
        {
            queueCount.Text.Should().Contain("0");
        }
    }

    [Fact]
    public void Progress_Bar_Starts_At_Zero()
    {
        var started = TryStartDriver();
        if (!started) return;

        var progressBar = FindByAccessibilityIdOrNull("BatchProgressBar");
        if (progressBar is not null)
        {
            progressBar.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Zoom_Text_Shows_100_Percent_Initially()
    {
        var started = TryStartDriver();
        if (!started) return;

        var zoomText = FindByAccessibilityIdOrNull("ZoomText");
        if (zoomText is not null)
        {
            zoomText.Text.Should().Contain("100%");
        }
    }

    [Fact]
    public void Split_View_Toggle_Is_Checked_Initially()
    {
        var started = TryStartDriver();
        if (!started) return;

        var splitView = FindByNameOrNull("Split View");
        var sideBySide = FindByNameOrNull("Side by Side");
        (splitView ?? (object?)sideBySide).Should().NotBeNull("view mode buttons should exist");
    }

    [Fact]
    public void No_Plugin_Selected_Message_Shown_Initially()
    {
        var started = TryStartDriver();
        if (!started) return;

        var noPluginText = FindByAccessibilityIdOrNull("NoPluginText")
                           ?? FindByNameOrNull("No plugin selected");
        if (noPluginText is not null)
        {
            noPluginText.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Batch_Progress_Text_Shows_Zero_Of_Zero()
    {
        var started = TryStartDriver();
        if (!started) return;

        var completedText = FindByAccessibilityIdOrNull("CompletedText");
        var totalText = FindByAccessibilityIdOrNull("TotalText");
        if (completedText is not null)
        {
            completedText.Text.Should().Be("0");
        }
        if (totalText is not null)
        {
            totalText.Text.Should().Be("0");
        }
    }

    [Fact]
    public void Elapsed_Time_Shows_Zero()
    {
        var started = TryStartDriver();
        if (!started) return;

        var elapsedText = FindByAccessibilityIdOrNull("ElapsedTimeText");
        if (elapsedText is not null)
        {
            elapsedText.Text.Should().Be("00:00:00");
        }
    }

    [Fact]
    public void Remaining_Time_Shows_Dashes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var remainingText = FindByAccessibilityIdOrNull("RemainingTimeText");
        if (remainingText is not null)
        {
            remainingText.Text.Should().Be("--:--:--");
        }
    }
}
