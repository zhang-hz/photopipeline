# Quick crash check script
Write-Host "=== Checking crash logs ==="
$logDir = "$env:LOCALAPPDATA\Photopipeline\logs"
if (Test-Path $logDir) {
    Get-ChildItem "$logDir\crash_*.log" | Sort-Object LastWriteTime -Descending | ForEach-Object {
        Write-Host "--- $($_.Name) ($($_.LastWriteTime)) ---"
        Get-Content $_.FullName
    }
} else {
    Write-Host "No log directory at $logDir"
}

Write-Host ""
Write-Host "=== Checking recent Application Errors (last 30 min) ==="
$since = (Get-Date).AddMinutes(-30)
Get-WinEvent -LogName Application -MaxEvents 50 -ErrorAction SilentlyContinue | Where-Object {
    $_.ProviderName -eq 'Application Error' -and $_.TimeCreated -gt $since
} | ForEach-Object {
    Write-Host "Time: $($_.TimeCreated) | ID: $($_.Id)"
    Write-Host $_.Message
    Write-Host "---"
}

Write-Host ""
Write-Host "=== Checking Windows Error Reporting (last 30 min) ==="
Get-WinEvent -LogName Application -MaxEvents 100 -ErrorAction SilentlyContinue | Where-Object {
    $_.Id -eq 1000 -and $_.TimeCreated -gt $since
} | ForEach-Object {
    Write-Host "Time: $($_.TimeCreated)"
    Write-Host $_.Message
}
