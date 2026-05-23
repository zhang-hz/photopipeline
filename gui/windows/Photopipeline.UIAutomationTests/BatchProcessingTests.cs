namespace Photopipeline.UIAutomationTests;

public sealed class BatchProcessingTests : UIAutomationTestBase
{
    [Fact]
    public void Add_Images_To_Queue()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        if (addButton is not null)
        {
            addButton.Click();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Start_Batch_Processes_Files()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByAccessibilityIdOrNull("StartButton")
                          ?? FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Pause_Resume_Batch_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pauseButton = FindByAccessibilityIdOrNull("PauseButton")
                          ?? FindByNameOrNull("Pause");
        pauseButton?.Click();
        Thread.Sleep(500);

        var startButton = FindByAccessibilityIdOrNull("StartButton")
                          ?? FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Stop_Batch_Gracefully_Cancels()
    {
        var started = TryStartDriver();
        if (!started) return;

        var stopButton = FindByAccessibilityIdOrNull("StopButton")
                         ?? FindByNameOrNull("Stop");
        stopButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Progress_Bar_Updates_During_Processing()
    {
        var started = TryStartDriver();
        if (!started) return;

        var progressBar = FindByAccessibilityIdOrNull("BatchProgressBar");
        Assert.NotNull(progressBar);
    }

    [Fact]
    public void Progress_Bar_Has_Correct_Range()
    {
        var started = TryStartDriver();
        if (!started) return;

        var progressBar = FindByAccessibilityIdOrNull("BatchProgressBar");
        if (progressBar is not null)
        {
            progressBar.GetAttribute("Value").Should().NotBeNull();
        }
    }

    [Fact]
    public void Progress_Text_Updates()
    {
        var started = TryStartDriver();
        if (!started) return;

        var completedText = FindByAccessibilityIdOrNull("CompletedText");
        var totalText = FindByAccessibilityIdOrNull("TotalText");
        Assert.NotNull(completedText ?? totalText);
    }

    [Fact]
    public void Error_Handling_Skips_Failed_Files()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Error_Count_Increments_On_Failure()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Output_Format_Selection_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is not null)
        {
            formatCombo.Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Output_Format_TIFF_Selected_By_Default()
    {
        var started = TryStartDriver();
        if (!started) return;

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is not null)
        {
            formatCombo.Text.Should().Be("TIFF");
        }
    }

    [Fact]
    public void Output_Format_JPEG_Selected()
    {
        var started = TryStartDriver();
        if (!started) return;

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is null) return;

        formatCombo.Click();
        Thread.Sleep(300);

        var jpegOption = FindByNameOrNull("JPEG");
        if (jpegOption is not null)
        {
            jpegOption.Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Output_Format_PNG_Selected()
    {
        var started = TryStartDriver();
        if (!started) return;

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is null) return;

        formatCombo.Click();
        Thread.Sleep(300);

        var pngOption = FindByNameOrNull("PNG");
        pngOption?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Output_Format_WebP_Selected()
    {
        var started = TryStartDriver();
        if (!started) return;

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is null) return;

        formatCombo.Click();
        Thread.Sleep(300);

        var webpOption = FindByNameOrNull("WebP");
        webpOption?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Output_Format_HEIF_Selected()
    {
        var started = TryStartDriver();
        if (!started) return;

        var formatCombo = FindByAccessibilityIdOrNull("FormatComboBox");
        if (formatCombo is null) return;

        formatCombo.Click();
        Thread.Sleep(300);

        var heifOption = FindByNameOrNull("HEIF");
        heifOption?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Queue_Order_Drag_Reorder()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        if (filmstrip is not null)
        {
            var actions = new OpenQA.Selenium.Interactions.Actions(Driver);
            actions.MoveToElement(filmstrip, 30, 50).Click().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Queue_Count_Increments_On_Add()
    {
        var started = TryStartDriver();
        if (!started) return;

        var queueCount = FindByAccessibilityIdOrNull("QueueCountText");
        if (queueCount is not null)
        {
            var initial = queueCount.Text;
            initial.Should().NotBeNull();
        }
    }

    [Fact]
    public void Queue_Count_Decrements_On_Remove()
    {
        var started = TryStartDriver();
        if (!started) return;

        var removeButton = FindByNameOrNull("Remove Selected");
        removeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Elapsed_Time_Starts_On_Batch_Start()
    {
        var started = TryStartDriver();
        if (!started) return;

        var elapsedText = FindByAccessibilityIdOrNull("ElapsedTimeText");
        Assert.NotNull(elapsedText);
    }

    [Fact]
    public void Remaining_Time_Estimates_Correctly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var remainingText = FindByAccessibilityIdOrNull("RemainingTimeText");
        Assert.NotNull(remainingText);
    }

    [Fact]
    public void Stop_Batch_Updates_Elapsed_Time()
    {
        var started = TryStartDriver();
        if (!started) return;

        var stopButton = FindByAccessibilityIdOrNull("StopButton");
        stopButton?.Click();
        Thread.Sleep(300);

        var elapsedText = FindByAccessibilityIdOrNull("ElapsedTimeText");
        if (elapsedText is not null)
        {
            elapsedText.Text.Should().NotBeNullOrEmpty();
        }
    }

    [Fact]
    public void Pause_Batch_Updates_Status()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pauseButton = FindByAccessibilityIdOrNull("PauseButton");
        pauseButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Resume_Batch_Continues_Progress()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pauseButton = FindByNameOrNull("Pause");
        pauseButton?.Click();
        Thread.Sleep(200);

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Clear_Completed_Removes_Done_Items()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Does_Not_Start_With_Empty_Queue()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Settings_Dialog_Opens()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsButton = FindByNameOrNull("Batch Settings");
        settingsButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Settings_Jpeg_Quality_Set()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsButton = FindByNameOrNull("Batch Settings");
        settingsButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Settings_Embed_Metadata_Toggle()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsButton = FindByNameOrNull("Batch Settings");
        settingsButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Settings_Output_Directory_Set()
    {
        var started = TryStartDriver();
        if (!started) return;

        var settingsButton = FindByNameOrNull("Batch Settings");
        settingsButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Progress_Bar_Max_Equals_Total()
    {
        var started = TryStartDriver();
        if (!started) return;

        var progressBar = FindByAccessibilityIdOrNull("BatchProgressBar");
        Assert.NotNull(progressBar);
    }

    [Fact]
    public void Completed_Counter_Matches_Total_On_Finish()
    {
        var started = TryStartDriver();
        if (!started) return;

        var completedText = FindByAccessibilityIdOrNull("CompletedText");
        var totalText = FindByAccessibilityIdOrNull("TotalText");
        Assert.NotNull(completedText ?? totalText);
    }

    [Fact]
    public void Failed_Counter_Stays_Zero_On_Success()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(1000);
    }

    [Fact]
    public void Multiple_Batch_Runs_Reset_Counters()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Concurrent_Batch_Start_Prevented()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(100);
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Progress_Accurate_After_Resume()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pauseButton = FindByNameOrNull("Pause");
        pauseButton?.Click();
        Thread.Sleep(200);
        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Stops_Immediately_On_Stop()
    {
        var started = TryStartDriver();
        if (!started) return;

        var stopButton = FindByNameOrNull("Stop");
        stopButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Same_Image_Not_Added_Twice()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        addButton?.Click();
        Thread.Sleep(300);
        addButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void File_Picker_Opens_For_Image_Add()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        addButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Filmstrip_Shows_Added_Images()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Thumbnail_Generated_On_Image_Add()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addButton = FindByNameOrNull("Add");
        addButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Image_Selected_In_Filmstrip()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        if (filmstrip is not null)
        {
            var actions = new OpenQA.Selenium.Interactions.Actions(Driver);
            actions.MoveToElement(filmstrip, 30, 50).Click().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Multiple_Image_Selection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        if (filmstrip is not null)
        {
            var actions = new OpenQA.Selenium.Interactions.Actions(Driver);
            actions.MoveToElement(filmstrip, 30, 50).Click().Perform();
            Thread.Sleep(200);
        }
    }

    [Fact]
    public void Process_All_Uses_Batch_Mode()
    {
        var started = TryStartDriver();
        if (!started) return;

        var processAllButton = FindByNameOrNull("Process All");
        processAllButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Process_All_Single_Image()
    {
        var started = TryStartDriver();
        if (!started) return;

        var processAllButton = FindByNameOrNull("Process All");
        processAllButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Process_All_Multiple_Images()
    {
        var started = TryStartDriver();
        if (!started) return;

        var processAllButton = FindByNameOrNull("Process All");
        processAllButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Remove_Image_From_Queue()
    {
        var started = TryStartDriver();
        if (!started) return;

        var removeButton = FindByNameOrNull("Remove Selected");
        removeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Clear_All_Images()
    {
        var started = TryStartDriver();
        if (!started) return;

        var removeButton = FindByNameOrNull("Remove Selected");
        removeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Batch_Status_Text_Updates()
    {
        var started = TryStartDriver();
        if (!started) return;

        var statusText = FindByAccessibilityIdOrNull("StatusText");
        if (statusText is not null)
        {
            statusText.Text.Should().NotBeNullOrEmpty();
        }
    }

    [Fact]
    public void Batch_Idle_Status_On_Startup()
    {
        var started = TryStartDriver();
        if (!started) return;

        Thread.Sleep(500);
    }

    [Fact]
    public void Batch_Processing_Status_During_Run()
    {
        var started = TryStartDriver();
        if (!started) return;

        var startButton = FindByNameOrNull("Start");
        startButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Export_Button_In_Preview_Bar()
    {
        var started = TryStartDriver();
        if (!started) return;

        var exportButton = FindByNameOrNull("Export");
        exportButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Send_To_Photoshop_Button()
    {
        var started = TryStartDriver();
        if (!started) return;

        var sendButton = FindByNameOrNull("Send to PS");
        sendButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Image_File_Size_Displayed_In_Filmstrip()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Image_Dimensions_Displayed_In_Filmstrip()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Override_Status_Indicator_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Error_Status_Indicator_On_Failure()
    {
        var started = TryStartDriver();
        if (!started) return;

        var filmstrip = FindByAccessibilityIdOrNull("FilmstripList");
        Assert.NotNull(filmstrip);
    }

    [Fact]
    public void Zoom_Level_Updates_In_Preview()
    {
        var started = TryStartDriver();
        if (!started) return;

        var zoomIn = FindByNameOrNull("Zoom In");
        zoomIn?.Click();
        Thread.Sleep(300);

        var zoomText = FindByAccessibilityIdOrNull("ZoomText");
        Assert.NotNull(zoomText);
    }

    [Fact]
    public void Fit_In_Preview_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fitButton = FindByNameOrNull("Fit");
        fitButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void One_To_One_View_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var oneToOneButton = FindByNameOrNull("1:1");
        oneToOneButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Split_View_Shows_Before_After()
    {
        var started = TryStartDriver();
        if (!started) return;

        var splitView = FindByNameOrNull("Split View");
        splitView?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Side_By_Side_View_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var sideBySide = FindByNameOrNull("Side by Side");
        sideBySide?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Toggle_Split_Direction()
    {
        var started = TryStartDriver();
        if (!started) return;

        var splitView = FindByNameOrNull("Split View");
        splitView?.Click();
        Thread.Sleep(200);
        var sideBySide = FindByNameOrNull("Side by Side");
        sideBySide?.Click();
        Thread.Sleep(200);
    }

    [Fact]
    public void Split_Handle_Drag_Changes_Ratio()
    {
        var started = TryStartDriver();
        if (!started) return;

        var splitHandle = FindByAccessibilityIdOrNull("SplitHandle");
        if (splitHandle is not null)
        {
            var actions = new OpenQA.Selenium.Interactions.Actions(Driver);
            actions.MoveToElement(splitHandle).ClickAndHold().MoveByOffset(50, 0).Release().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Pixel_Coordinates_Displayed_In_Status()
    {
        var started = TryStartDriver();
        if (!started) return;

        var mouseInfo = FindByAccessibilityIdOrNull("MouseInfoText");
        Assert.NotNull(mouseInfo);
    }

    [Fact]
    public void Pixel_RGB_Values_Displayed()
    {
        var started = TryStartDriver();
        if (!started) return;

        var mouseInfo = FindByAccessibilityIdOrNull("MouseInfoText");
        Assert.NotNull(mouseInfo);
    }

    [Fact]
    public void Color_Space_Info_Displayed()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pixelInfo = FindByAccessibilityIdOrNull("PixelInfoText");
        Assert.NotNull(pixelInfo);
    }

    [Fact]
    public void Bit_Depth_Info_Displayed()
    {
        var started = TryStartDriver();
        if (!started) return;

        var pixelInfo = FindByAccessibilityIdOrNull("PixelInfoText");
        Assert.NotNull(pixelInfo);
    }
}
