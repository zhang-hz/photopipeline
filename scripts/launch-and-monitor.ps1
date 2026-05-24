param(
    [int]$WaitSeconds = 5
)

$exe = "c:\Data\code\photopipeline\staging\Photopipeline.exe"
$marker = Get-Date
$logFile = "c:\Data\code\photopipeline\scripts\launch-result.txt"

$sb = [System.Text.StringBuilder]::new()
[void]$sb.AppendLine("Launching: $exe")
[void]$sb.AppendLine("Marker time: $($marker.ToString('HH:mm:ss'))")

$proc = Start-Process -FilePath $exe -PassThru
[void]$sb.AppendLine("Process ID: $($proc.Id)")

Start-Sleep -Seconds $WaitSeconds

$proc.Refresh()
if ($proc.HasExited) {
    [void]$sb.AppendLine("CRASHED! Exit code: $($proc.ExitCode), Time: $($proc.ExitTime)")
} else {
    [void]$sb.AppendLine("Still running after ${WaitSeconds}s - NO crash!")
    $proc.Kill()
    [void]$sb.AppendLine("Killed process.")
}

[void]$sb.AppendLine("")
[void]$sb.AppendLine("=== New crash events since marker ===")
$events = Get-WinEvent -LogName Application -MaxEvents 20 -ErrorAction SilentlyContinue | Where-Object {
    $_.Id -eq 1000 -and $_.TimeCreated -gt $marker
}
if ($events) {
    foreach ($e in $events) {
        [void]$sb.AppendLine("Time: $($e.TimeCreated)")
        [void]$sb.AppendLine($e.Message)
        [void]$sb.AppendLine("---")
    }
} else {
    [void]$sb.AppendLine("No new crash events!")
}

$sb.ToString() | Out-File -FilePath $logFile -Encoding UTF8
Write-Output $sb.ToString()
