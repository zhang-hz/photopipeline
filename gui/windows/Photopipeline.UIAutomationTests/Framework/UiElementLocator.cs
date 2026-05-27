using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Conditions;
using FlaUI.Core.Definitions;
using FlaUI.Core.Identifiers;
using FlaUI.Core.Tools;
using FlaUI.UIA3;

namespace Photopipeline.UIAutomationTests.Framework;

/// <summary>
/// Provides reliable element location strategies for WPF UI automation.
/// Searches are ordered by reliability: AutomationId > Name > ClassName.
/// </summary>
public static class UiElementLocator
{
    private static readonly TimeSpan DefaultTimeout = TimeSpan.FromSeconds(30);
    private static readonly TimeSpan PollInterval = TimeSpan.FromMilliseconds(200);

    // ── Strategy 1: By AutomationId (most reliable for WPF) ──

    /// <summary>
    /// Finds a descendant element by its AutomationProperties.AutomationId.
    /// Returns null if not found within the timeout.
    /// </summary>
    public static AutomationElement? ByAutomationId(
        AutomationElement parent,
        string automationId,
        TimeSpan? timeout = null)
    {
        return WaitForElement(
            parent,
            cf => cf.ByAutomationId(automationId),
            timeout ?? DefaultTimeout);
    }

    /// <summary>
    /// Finds a descendant element by AutomationId. Throws if not found.
    /// </summary>
    public static AutomationElement RequireByAutomationId(
        AutomationElement parent,
        string automationId,
        TimeSpan? timeout = null)
    {
        var element = ByAutomationId(parent, automationId, timeout);
        if (element == null)
            throw new InvalidOperationException(
                $"Required element with AutomationId '{automationId}' not found within {timeout ?? DefaultTimeout}");
        return element;
    }

    // ── Strategy 2: By Name ──

    /// <summary>
    /// Finds a descendant element by its Name property (exact match).
    /// </summary>
    public static AutomationElement? ByName(
        AutomationElement parent,
        string name,
        TimeSpan? timeout = null)
    {
        return WaitForElement(
            parent,
            cf => cf.ByName(name),
            timeout ?? DefaultTimeout);
    }

    /// <summary>
    /// Finds a descendant element by Name. Throws if not found.
    /// </summary>
    public static AutomationElement RequireByName(
        AutomationElement parent,
        string name,
        TimeSpan? timeout = null)
    {
        var element = ByName(parent, name, timeout);
        if (element == null)
            throw new InvalidOperationException(
                $"Required element with Name '{name}' not found within {timeout ?? DefaultTimeout}");
        return element;
    }

    /// <summary>
    /// Finds a descendant element whose Name contains the given substring.
    /// </summary>
    public static AutomationElement? ByNameContains(
        AutomationElement parent,
        string substring,
        TimeSpan? timeout = null)
    {
        var effectiveTimeout = timeout ?? DefaultTimeout;
        var deadline = DateTime.UtcNow + effectiveTimeout;

        while (DateTime.UtcNow < deadline)
        {
            var element = parent.FindFirstDescendant(cf =>
                cf.ByControlType(ControlType.Text)
                    .Or(cf.ByControlType(ControlType.Button))
                    .Or(cf.ByControlType(ControlType.ListItem))
                    .Or(cf.ByControlType(ControlType.TabItem))
                    .Or(cf.ByControlType(ControlType.Hyperlink)));

            // Fallback: search all descendants and filter by Name
            var allDescendants = FindAllDescendantsBroad(parent);
            foreach (var desc in allDescendants)
            {
                if (!string.IsNullOrEmpty(desc.Name) && desc.Name.Contains(substring, StringComparison.OrdinalIgnoreCase))
                    return desc;
            }

            Thread.Sleep(PollInterval);
        }
        return null;
    }

    // ── Strategy 3: By ClassName ──

    /// <summary>
    /// Finds a descendant element by its ClassName.
    /// </summary>
    public static AutomationElement? ByClassName(
        AutomationElement parent,
        string className,
        TimeSpan? timeout = null)
    {
        return WaitForElement(
            parent,
            cf => cf.ByClassName(className),
            timeout ?? DefaultTimeout);
    }

    // ── Strategy 4: By ControlType ──

    /// <summary>
    /// Finds the first descendant of a given ControlType.
    /// </summary>
    public static AutomationElement? ByControlType(
        AutomationElement parent,
        ControlType controlType,
        TimeSpan? timeout = null)
    {
        return WaitForElement(
            parent,
            cf => cf.ByControlType(controlType),
            timeout ?? DefaultTimeout);
    }

    // ── Strategy 5: By ControlType + Name ──

    /// <summary>
    /// Finds a descendant by ControlType whose Name contains a substring.
    /// </summary>
    public static AutomationElement? ByTypeAndName(
        AutomationElement parent,
        ControlType controlType,
        string nameSubstring,
        TimeSpan? timeout = null)
    {
        var effectiveTimeout = timeout ?? DefaultTimeout;
        var deadline = DateTime.UtcNow + effectiveTimeout;

        while (DateTime.UtcNow < deadline)
        {
            var allOfType = parent.FindAllDescendants(cf => cf.ByControlType(controlType));
            foreach (var elem in allOfType)
            {
                if (!string.IsNullOrEmpty(elem.Name) &&
                    elem.Name.Contains(nameSubstring, StringComparison.OrdinalIgnoreCase))
                {
                    return elem;
                }
            }
            Thread.Sleep(PollInterval);
        }
        return null;
    }

    // ── Utility: Wait for element ──

    /// <summary>
    /// Repeatedly searches for an element matching the given condition until found or timeout.
    /// </summary>
    public static AutomationElement? WaitForElement(
        AutomationElement parent,
        Func<ConditionFactory, ConditionBase> conditionBuilder,
        TimeSpan timeout)
    {
        var automation = parent.Automation;
        var cf = automation.ConditionFactory;
        var condition = conditionBuilder(cf);

        var result = Retry.WhileNull(
            () => parent.FindFirstDescendant(condition),
            timeout,
            PollInterval);

        return result.Result;
    }

    /// <summary>
    /// Finds all descendants that match any common WPF ControlType.
    /// Used as a fallback for broad element searches.
    /// </summary>
    public static AutomationElement[] FindAllDescendantsBroad(AutomationElement parent)
    {
        var cf = parent.Automation.ConditionFactory;
        var broadCondition = BuildBroadCondition(cf);
        return parent.FindAllDescendants(broadCondition);
    }

    /// <summary>
    /// Builds a condition that matches most common WPF ControlTypes.
    /// </summary>
    private static ConditionBase BuildBroadCondition(ConditionFactory cf)
    {
        return cf.ByControlType(ControlType.Button)
            .Or(cf.ByControlType(ControlType.Text))
            .Or(cf.ByControlType(ControlType.Edit))
            .Or(cf.ByControlType(ControlType.List))
            .Or(cf.ByControlType(ControlType.ListItem))
            .Or(cf.ByControlType(ControlType.ComboBox))
            .Or(cf.ByControlType(ControlType.CheckBox))
            .Or(cf.ByControlType(ControlType.TabItem))
            .Or(cf.ByControlType(ControlType.Group))
            .Or(cf.ByControlType(ControlType.Pane))
            .Or(cf.ByControlType(ControlType.Hyperlink))
            .Or(cf.ByControlType(ControlType.Image))
            .Or(cf.ByControlType(ControlType.Slider))
            .Or(cf.ByControlType(ControlType.ProgressBar))
            .Or(cf.ByControlType(ControlType.Tree))
            .Or(cf.ByControlType(ControlType.TreeItem))
            .Or(cf.ByControlType(ControlType.MenuItem))
            .Or(cf.ByControlType(ControlType.Custom))
            .Or(cf.ByControlType(ControlType.Header))
            .Or(cf.ByControlType(ControlType.Document))
            .Or(cf.ByControlType(ControlType.ScrollBar))
            .Or(cf.ByControlType(ControlType.StatusBar))
            .Or(cf.ByControlType(ControlType.TitleBar))
            .Or(cf.ByControlType(ControlType.Menu))
            .Or(cf.ByControlType(ControlType.Window));
    }

    /// <summary>
    /// Checks whether a descendant element with the given AutomationId exists.
    /// </summary>
    public static bool Exists(AutomationElement parent, string automationId)
    {
        return ByAutomationId(parent, automationId, TimeSpan.FromMilliseconds(500)) != null;
    }

    // ── Screenshot on failure ──

    /// <summary>
    /// Captures a screenshot of the given element to the test screenshots directory.
    /// Returns the file path of the saved screenshot.
    /// </summary>
    public static string CaptureScreenshot(AutomationElement element, string? testName = null)
    {
        var screenshotDir = Path.Combine(
            Path.GetTempPath(), "photopipeline_tests", "screenshots");
        Directory.CreateDirectory(screenshotDir);

        var fileName = $"{testName ?? "screenshot"}_{DateTime.Now:yyyyMMdd_HHmmss_fff}.png";
        var filePath = Path.Combine(screenshotDir, fileName);

        try
        {
            var capture = element.Capture();
            if (capture != null)
            {
                capture.Save(filePath, System.Drawing.Imaging.ImageFormat.Png);
                capture.Dispose();
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Screenshot capture failed: {ex.Message}");
            // Best-effort: write a placeholder file with error info
            File.WriteAllText(filePath + ".error.txt", $"Failed to capture screenshot: {ex}");
        }

        return filePath;
    }
}
