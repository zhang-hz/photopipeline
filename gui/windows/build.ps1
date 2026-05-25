# Builds the WPF GUI project (used by CI and local dev)
param(
    [string]$Configuration = "Release",
    [string]$Architecture = "x64",
    [string]$Runtime = "win-x64",
    [string]$OutputDir = "",
    [switch]$SelfContained = $true,
    [switch]$SkipTests = $false,
    [switch]$SkipPublish = $false,
    [switch]$CreateStaging = $false
)

$ErrorActionPreference = "Stop"
$env:Path = "C:\Program Files\dotnet;$env:Path"

if (-not $OutputDir) {
    $OutputDir = "$PSScriptRoot\publish"
}

$RepoRoot = Resolve-Path "$PSScriptRoot\..\.."
$StagingDir = "$RepoRoot\staging"
$BackendPath = "$RepoRoot\target\release\photopipeline-server.exe"
$CliPath = "$RepoRoot\target\release\photopipeline-cli.exe"

Write-Host "=== Photopipeline WPF Build ===" -ForegroundColor Cyan
Write-Host "Configuration : $Configuration"
Write-Host "Architecture  : $Architecture"
Write-Host "Runtime       : $Runtime"
Write-Host "Output        : $OutputDir"
Write-Host "SelfContained : $SelfContained"
Write-Host ""

# ── Step 1: Restore ──────────────────────────────────────────
Write-Host "[1/5] Restoring packages..." -ForegroundColor Yellow
& dotnet restore Photopipeline/Photopipeline.csproj -a $Architecture
if ($LASTEXITCODE -ne 0) {
    Write-Host "Package restore failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "  -> OK" -ForegroundColor Green
Write-Host ""

# ── Step 2: Build ────────────────────────────────────────────
Write-Host "[2/5] Building solution..." -ForegroundColor Yellow
& dotnet build Photopipeline.sln -c $Configuration -p:Platform=$Architecture
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "  -> OK" -ForegroundColor Green
Write-Host ""

# ── Step 3: Test ─────────────────────────────────────────────
if (-not $SkipTests) {
    Write-Host "[3/5] Running tests..." -ForegroundColor Yellow
    & dotnet test Photopipeline.Tests/Photopipeline.Tests.csproj -c $Configuration -p:Platform=$Architecture --no-build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Tests failed!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "  -> OK" -ForegroundColor Green
} else {
    Write-Host "[3/5] Tests: SKIPPED" -ForegroundColor DarkGray
}
Write-Host ""

# ── Step 4: Publish ──────────────────────────────────────────
if (-not $SkipPublish) {
    Write-Host "[4/5] Publishing GUI (self-contained single-file)..." -ForegroundColor Yellow
    $publishArgs = @(
        "publish", "Photopipeline/Photopipeline.csproj",
        "-c", $Configuration,
        "-p:Platform=$Architecture",
        "-r", $Runtime,
        "-o", $OutputDir
    )
    if ($SelfContained) {
        $publishArgs += "--self-contained"
        $publishArgs += "-p:WindowsAppSDKSelfContained=true"
    }
    & dotnet $publishArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Publish failed!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "  -> OK" -ForegroundColor Green
} else {
    Write-Host "[4/5] Publish: SKIPPED" -ForegroundColor DarkGray
}
Write-Host ""

# ── Step 5: Stage ────────────────────────────────────────────
if ($CreateStaging) {
    Write-Host "[5/5] Creating staging directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path $StagingDir | Out-Null

    # Copy GUI publish output
    Copy-Item "$OutputDir\*" "$StagingDir\" -Recurse -Force
    Write-Host "  Copied GUI from $OutputDir"

    # Copy Rust backend (if built)
    if (Test-Path $BackendPath) {
        Copy-Item $BackendPath "$StagingDir\"
        Write-Host "  Copied backend: photopipeline-server.exe"
    } else {
        Write-Host "  Backend not found (skipped): $BackendPath" -ForegroundColor DarkYellow
    }

    # Copy CLI (if built)
    if (Test-Path $CliPath) {
        Copy-Item $CliPath "$StagingDir\"
        Write-Host "  Copied CLI: photopipeline-cli.exe"
    }

    # Copy license
    $LicensePath = "$RepoRoot\LICENSE"
    if (Test-Path $LicensePath) {
        Copy-Item $LicensePath "$StagingDir\license.txt"
        Write-Host "  Copied LICENSE"
    }

    # Output manifest
    $fileCount = (Get-ChildItem $StagingDir -Recurse | Measure-Object).Count
    Write-Host ""
    Write-Host "  Staging directory: $StagingDir" -ForegroundColor Cyan
    Write-Host "  Files staged: $fileCount" -ForegroundColor Cyan
} else {
    Write-Host "[5/5] Stage: SKIPPED" -ForegroundColor DarkGray
}
Write-Host ""

Write-Host "Build complete." -ForegroundColor Green
if ($CreateStaging) {
    Write-Host "Output: $StagingDir" -ForegroundColor Green
} else {
    Write-Host "Output: $OutputDir" -ForegroundColor Green
}
