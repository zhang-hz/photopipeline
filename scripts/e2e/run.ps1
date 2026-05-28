# Photopipeline E2E Test Runner
# Builds the binary, packages DLLs, runs all 520+ tests sequentially.
param(
    [string]$BinaryPath = "dist/photopipeline.exe",
    [string]$OutputDir = "tests/e2e_suite/output",
    [string]$ReportFile = "e2e_report.json",
    [string]$Categories = "all"
)

$ErrorActionPreference = "Stop"

Write-Host "=== Photopipeline E2E Test Suite ===" -ForegroundColor Cyan

# Step 1: Build the server binary with all features
Write-Host "`n[1/4] Building photopipeline server..." -ForegroundColor Yellow
cargo build --release -p photopipeline-server --features "libheif-native,libjxl-native,libraw-native,lcms2-native"
if ($LASTEXITCODE -ne 0) { Write-Error "Build failed"; exit 1 }

# Step 2: Build the E2E runner
Write-Host "`n[2/4] Building E2E test runner..." -ForegroundColor Yellow
cargo build --release -p photopipeline-e2e-runner
if ($LASTEXITCODE -ne 0) { Write-Error "Build failed"; exit 1 }

# Step 3: Package (copy binary + DLLs to dist/)
Write-Host "`n[3/4] Packaging..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path dist | Out-Null
Copy-Item "target/release/photopipeline.exe" "dist/" -Force

# Copy vcpkg DLLs if available
if ($env:VCPKG_ROOT) {
    $vcpkgBin = Join-Path $env:VCPKG_ROOT "installed/x64-windows/bin"
    if (Test-Path $vcpkgBin) {
        Copy-Item "$vcpkgBin/*.dll" "dist/" -ErrorAction SilentlyContinue
        Write-Host "  Copied vcpkg DLLs from $vcpkgBin"
    }
}
# Copy vendor DLLs (patched libjxl/libraw)
$vendorDll = "target/release/build"
Get-ChildItem -Path $vendorDll -Recurse -Filter "*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
    Copy-Item $_.FullName "dist/" -Force -ErrorAction SilentlyContinue
}

# Verify binary exists
if (-not (Test-Path $BinaryPath)) {
    Copy-Item "target/release/photopipeline.exe" $BinaryPath -Force
}
Write-Host "  Binary: $BinaryPath"

# Step 4: Run E2E tests (serial, one at a time)
Write-Host "`n[4/4] Running E2E tests..." -ForegroundColor Yellow
$runner = "target/release/photopipeline-e2e-runner.exe"
if (-not (Test-Path $runner)) {
    $runner = "target/debug/photopipeline-e2e-runner.exe"
}

& $runner --binary $BinaryPath --output-dir $OutputDir --report $ReportFile --categories $Categories

Write-Host "`n=== E2E Test Suite Complete ===" -ForegroundColor Cyan
Write-Host "Report: $ReportFile"
Write-Host "Output images: $OutputDir"

# Show summary from report
if (Test-Path $ReportFile) {
    $report = Get-Content $ReportFile | ConvertFrom-Json
    Write-Host "Total: $($report.total)"
    Write-Host "Passed: $($report.passed)"
    Write-Host "Bypass: $($report.failed_bypass)"
    Write-Host "Failed: $($report.failed_real)"
    Write-Host "Timeout: $($report.timed_out)"
}
