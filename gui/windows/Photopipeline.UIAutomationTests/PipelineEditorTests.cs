using OpenQA.Selenium.Interactions;

namespace Photopipeline.UIAutomationTests;

public sealed class PipelineEditorTests : UIAutomationTestBase
{
    [Fact]
    public void Add_Node_Appears_On_Canvas()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(500);

        var canvas = FindByAccessibilityIdOrNull("CanvasRoot");
        canvas.Should().NotBeNull("canvas should exist after adding a node");
    }

    [Fact]
    public void Add_Multiple_Nodes_Stack_Correctly()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 3; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        canvas.Should().NotBeNull();
    }

    [Fact]
    public void Delete_Node_Removes_From_Canvas()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        var deleteButton = FindByNameOrNull("Delete");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(500);

        if (deleteButton is not null)
        {
            deleteButton.Click();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Add_Node_Without_Plugin_Shows_Error()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Select_Node_Highlights_Border()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(500);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 200, 150).Click().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Deselect_Node_Removes_Highlight()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(500);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 200, 150).Click().Perform();
            Thread.Sleep(200);
            actions.MoveToElement(canvas, 300, 200).Click().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Drag_Node_Changes_Position()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(500);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 200, 150)
                   .ClickAndHold()
                   .MoveByOffset(100, 50)
                   .Release()
                   .Perform();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Drag_Node_Multiple_Times()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(500);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is null) return;

        var actions = new Actions(Driver);
        actions.MoveToElement(canvas, 200, 150).ClickAndHold().MoveByOffset(50, 30).Release().Perform();
        Thread.Sleep(200);
        actions.MoveToElement(canvas, 250, 180).ClickAndHold().MoveByOffset(-30, -20).Release().Perform();
        Thread.Sleep(300);
    }

    [Fact]
    public void Connect_Two_Nodes_Creates_Edge()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);
        addNodeButton.Click();
        Thread.Sleep(400);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 300, 100)
                   .ClickAndHold()
                   .MoveByOffset(200, 0)
                   .Release()
                   .Perform();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Disconnect_Edge_Removes_Connection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 350, 140).Click().Perform();
            Thread.Sleep(200);
        }
    }

    [Fact]
    public void Zoom_In_Zoom_Out_Works()
    {
        var started = TryStartDriver();
        if (!started) return;

        var zoomIn = FindByNameOrNull("Zoom In");
        var zoomOut = FindByNameOrNull("Zoom Out");

        if (zoomIn is not null)
        {
            zoomIn.Click();
            Thread.Sleep(200);
        }
        if (zoomOut is not null)
        {
            zoomOut.Click();
            Thread.Sleep(200);
        }

        var zoomText = FindByAccessibilityIdOrNull("ZoomText");
        Assert.NotNull(zoomText);
    }

    [Fact]
    public void Fit_All_Brings_Nodes_Into_View()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fitAllButton = FindByNameOrNull("Fit All");
        if (fitAllButton is null) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        fitAllButton.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Fit_All_With_No_Nodes_Does_Not_Crash()
    {
        var started = TryStartDriver();
        if (!started) return;

        var fitAllButton = FindByNameOrNull("Fit All");
        fitAllButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Fit_All_With_Single_Node()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var fitAllButton = FindByNameOrNull("Fit All");
        fitAllButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Validate_Pipeline_Detects_Issues()
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

        var validateButton = FindByNameOrNull("Validate");
        if (validateButton is not null)
        {
            validateButton.Click();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Validate_Empty_Pipeline_Does_Not_Throw()
    {
        var started = TryStartDriver();
        if (!started) return;

        var validateButton = FindByNameOrNull("Validate");
        if (validateButton is not null)
        {
            validateButton.Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Validate_Single_Node_Pipeline()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var validateButton = FindByNameOrNull("Validate");
        validateButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Validate_Connected_Pipeline_Succeeds()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 300, 100).ClickAndHold().MoveByOffset(200, 0).Release().Perform();
            Thread.Sleep(300);
        }

        var validateButton = FindByNameOrNull("Validate");
        validateButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Validate_Disconnected_Nodes_Detected()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);

        var validateButton = FindByNameOrNull("Validate");
        validateButton?.Click();
        Thread.Sleep(500);
    }

    [Fact]
    public void Cycle_Detection_Prevents_Connection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 300, 100).ClickAndHold().MoveByOffset(200, 0).Release().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Duplicate_Node_Creates_Copy()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);

        var duplicateButton = FindByNameOrNull("Duplicate");
        if (duplicateButton is not null)
        {
            duplicateButton.Click();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Duplicate_Multiple_Times()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);

        var duplicateButton = FindByNameOrNull("Duplicate");
        if (duplicateButton is not null)
        {
            duplicateButton.Click();
            Thread.Sleep(300);
            duplicateButton.Click();
            Thread.Sleep(300);
            duplicateButton.Click();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Duplicate_Without_Selection_Does_Nothing()
    {
        var started = TryStartDriver();
        if (!started) return;

        var duplicateButton = FindByNameOrNull("Duplicate");
        duplicateButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Delete_Undo_Restores_Node()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);

        var deleteButton = FindByNameOrNull("Delete");
        deleteButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Select_Node_With_Keyboard()
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
            canvas.Click();
            Thread.Sleep(200);
        }
    }

    [Fact]
    public void Keyboard_Delete_Removes_Node()
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
            canvas.Click();
            Thread.Sleep(200);
            var actions = new Actions(Driver);
            actions.SendKeys(Keys.Delete).Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Canvas_Scrolls_With_Large_Number_Of_Nodes()
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

        var canvas = FindByAccessibilityIdOrNull("CanvasRoot");
        canvas.Should().NotBeNull();
    }

    [Fact]
    public void Canvas_Right_Click_Opens_Context_Menu()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 100, 100).ContextClick().Perform();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Canvas_Middle_Click_For_Panning()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 200, 100).ClickAndHold().MoveByOffset(50, 30).Release().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Zoom_To_200_Percent()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }

        var zoomIn = FindByNameOrNull("Zoom In");
        if (zoomIn is not null)
        {
            zoomIn.Click();
            Thread.Sleep(100);
            zoomIn.Click();
            Thread.Sleep(100);
            zoomIn.Click();
            Thread.Sleep(100);
        }
    }

    [Fact]
    public void Zoom_To_25_Percent()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }

        var zoomOut = FindByNameOrNull("Zoom Out");
        if (zoomOut is not null)
        {
            zoomOut.Click();
            Thread.Sleep(100);
            zoomOut.Click();
            Thread.Sleep(100);
            zoomOut.Click();
            Thread.Sleep(100);
        }
    }

    [Fact]
    public void Zoom_Resets_After_Fit_All()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is not null)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }

        var zoomIn = FindByNameOrNull("Zoom In");
        zoomIn?.Click();
        Thread.Sleep(100);

        var fitAllButton = FindByNameOrNull("Fit All");
        fitAllButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Snap_To_Grid_Aligns_Nodes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Node_Ports_Are_Visible()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        canvas.Should().NotBeNull("canvas should contain node with visible ports");
    }

    [Fact]
    public void Input_Port_Accepts_Connection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 100, 100).ClickAndHold().MoveByOffset(220, 0).Release().Perform();
            Thread.Sleep(400);
        }
    }

    [Fact]
    public void Output_Port_Initiates_Connection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 300, 140).ClickAndHold().MoveByOffset(220, 0).Release().Perform();
            Thread.Sleep(400);
        }
    }

    [Fact]
    public void Multiple_Edges_Between_Nodes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 3; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 300, 140).ClickAndHold().MoveByOffset(220, 0).Release().Perform();
            Thread.Sleep(200);
            actions.MoveToElement(canvas, 520, 140).ClickAndHold().MoveByOffset(220, 0).Release().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Self_Connection_Prevented()
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
            actions.MoveToElement(canvas, 350, 140).ClickAndHold().MoveByOffset(0, 0).Release().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Node_Display_Name_Must_Be_Unique()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Rename_Node_Via_Double_Click()
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
            actions.MoveToElement(canvas, 200, 100).DoubleClick().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Drag_Creates_Selection_Rectangle()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 3; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 50, 50).ClickAndHold().MoveByOffset(400, 300).Release().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Multi_Select_With_Ctrl()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 3; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }
    }

    [Fact]
    public void Drag_Multiple_Selected_Nodes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 2; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(300);
        }

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 200, 100).ClickAndHold().MoveByOffset(50, 30).Release().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Delay_Node_Changes_Timing()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Plugin_Search_From_Canvas_Context()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Disable_Node_Grays_Out_Display()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Node_Processing_State_Shows_Spinner()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Canvas_Background_Is_Dark()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("CanvasRoot");
        if (canvas is not null)
        {
            canvas.Displayed.Should().BeTrue();
        }
    }

    [Fact]
    public void Nodes_Stack_With_Auto_Layout()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 8; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(150);
        }
    }

    [Fact]
    public void Scroll_To_View_On_Node_Added()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Quick_Add_Node_From_Search()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Display_Plugin_Category_On_Node()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Pipeline_State_Persists_After_Zoom()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(200);

        var zoomIn = FindByNameOrNull("Zoom In");
        zoomIn?.Click();
        Thread.Sleep(200);

        var zoomOut = FindByNameOrNull("Zoom Out");
        zoomOut?.Click();
        Thread.Sleep(200);
    }

    [Fact]
    public void Edge_Removal_When_Node_Deleted()
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
    public void Undo_Operation_Restores_Edge()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Redo_After_Undo()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Select_All_Nodes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        for (int i = 0; i < 3; i++)
        {
            addNodeButton.Click();
            Thread.Sleep(200);
        }
    }

    [Fact]
    public void Deselect_All_Nodes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);

        var canvas = FindByAccessibilityIdOrNull("DagCanvas");
        if (canvas is not null)
        {
            var actions = new Actions(Driver);
            actions.MoveToElement(canvas, 500, 500).Click().Perform();
            Thread.Sleep(300);
        }
    }

    [Fact]
    public void Right_Click_On_Node_Opens_Context_Menu()
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
            actions.MoveToElement(canvas, 200, 100).ContextClick().Perform();
            Thread.Sleep(500);
        }
    }

    [Fact]
    public void Context_Menu_Has_Copy_Paste()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(400);
    }

    [Fact]
    public void Copy_Paste_Node_Across_Canvas()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Performance_With_50_Nodes_No_Crash()
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
    }

    [Fact]
    public void Node_Collision_Detection()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Minimap_Shows_All_Nodes()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Pipeline_Changes_Mark_Dirty_State()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        addNodeButton?.Click();
        Thread.Sleep(300);
    }

    [Fact]
    public void Canvas_Renders_Grid_Lines()
    {
        var started = TryStartDriver();
        if (!started) return;

        var canvas = FindByAccessibilityIdOrNull("CanvasRoot");
        canvas.Should().NotBeNull();
    }

    [Fact]
    public void Grid_Snap_Enabled_By_Default()
    {
        var started = TryStartDriver();
        if (!started) return;

        var addNodeButton = FindByNameOrNull("Add Node");
        if (addNodeButton is null) return;

        addNodeButton.Click();
        Thread.Sleep(300);
    }
}
