param(
    [switch]$Level1,            # Unit tests only (~30s)
    [switch]$Level2,            # Unit + Integration + gRPC (~3min)
    [switch]$Level3,            # Full: build + stage + all tests + E2E 520 (~12min)
    [switch]$Level4,            # Full + stress loops (~30min)
    [switch]$Clean,             # Clean build artifacts first
    [switch]$NoBuild,           # Skip build (use existing binaries)
    [string]$Features = "libheif-native,libjxl-native,libraw-native",
    [string]$OutputDir = "",
    [switch]$NoE2E              # Skip E2E even in Level3/4
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path "$ScriptDir\.."
if (-not $OutputDir) { $OutputDir = "$RepoRoot\staging" }
$env:VCPKG_ROOT = if (Test-Path "C:\vcpkg") { "C:\vcpkg" } else { $env:VCPKG_ROOT }

# Default: Level3 (full pipeline)
if (-not ($Level1 -or $Level2 -or $Level3 -or $Level4)) { $Level3 = $true }

Write-Host "============================================" -ForegroundColor Cyan
Write-Host " Photopipeline CI" -ForegroundColor Cyan
if ($Level1) { Write-Host " Level: 1 (Unit only)" -ForegroundColor Cyan }
if ($Level2) { Write-Host " Level: 2 (Unit + Integration)" -ForegroundColor Cyan }
if ($Level3) { Write-Host " Level: 3 (Full: Build + Stage + Tests + E2E)" -ForegroundColor Cyan }
if ($Level4) { Write-Host " Level: 4 (Full + Stress)" -ForegroundColor Cyan }
Write-Host " Features: $Features" -ForegroundColor Cyan
Write-Host " Output  : $OutputDir" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

function header([string]$text) { Write-Host $text -ForegroundColor Yellow }
function ok() { Write-Host "  -> OK" -ForegroundColor Green; Write-Host "" }
function fatal([string]$msg) { Write-Host "  -> FAIL: $msg" -ForegroundColor Red; exit 1 }
function warn([string]$msg) { Write-Host "  -> WARN: $msg" -ForegroundColor Magenta }
function info([string]$msg) { Write-Host "  $msg" -ForegroundColor Gray }

function sh([string]$cmd, [string]$cwd, [string]$err) {
    $full = if ($cwd) { "cd /d `"$cwd`" && $cmd 2>&1" } else { "$cmd 2>&1" }
    cmd /c $full
    if ($LASTEXITCODE -ne 0) { fatal $err }
}

function cargo([string]$cmd) {
    Write-Host "    cargo $cmd" -ForegroundColor DarkGray
    $full = "cargo $cmd 2>&1"
    cmd /c $full
    if ($LASTEXITCODE -ne 0) { fatal "cargo $cmd failed" }
}

# ── Clean ─────────────────────────────────────────────────────
if ($Clean) {
    header "[Clean] cargo clean"
    cargo "clean"
    ok
}

# ── Build + Stage (required for Level3/4) ─────────────────────
$needBuild = ($Level3 -or $Level4) -and -not $NoBuild
if ($needBuild) {
    header "[Build] cargo build --profile ci --workspace --features `"$Features`""
    cargo "build --profile ci --workspace --features `"$Features`""
    ok

    header "[Stage] Build E2E runner"
    cargo "build --profile ci -p photopipeline-e2e-runner"
    ok

    header "[Stage] Package binary + DLLs"
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
    Copy-Item "target/ci/photopipeline.exe" "$OutputDir/" -ErrorAction Stop
    Copy-Item "target/ci/photopipeline-e2e-runner.exe" "$OutputDir/" -ErrorAction SilentlyContinue

    $vcpkgBin = Join-Path $env:VCPKG_ROOT "installed/x64-windows/bin"
    if (Test-Path $vcpkgBin) {
        Copy-Item "$vcpkgBin/*.dll" "$OutputDir/" -ErrorAction SilentlyContinue
        info "Copied vcpkg DLLs from $vcpkgBin"
    }
    $requiredDlls = @("heif.dll", "jxl.dll", "raw.dll", "raw_r.dll", "lcms2-2.dll")
    $missingDlls = @()
    foreach ($dll in $requiredDlls) {
        if (-not (Test-Path "$OutputDir\$dll")) { $missingDlls += $dll }
        else { info "  DLL: $dll" }
    }
    if ($missingDlls.Count -gt 0) {
        warn "Missing DLLs: $missingDlls — install via vcpkg"
        Write-Host "  vcpkg install libheif libjxl libraw lcms2" -ForegroundColor Gray
    }
    if (Test-Path "LICENSE") { Copy-Item "LICENSE" "$OutputDir/license.txt" }
    info "Staged $(Get-ChildItem $OutputDir | Measure-Object | Select-Object -ExpandProperty Count) files to $OutputDir"

    header "[Smoke] $OutputDir\photopipeline.exe --version"
    sh "`"$OutputDir\photopipeline.exe`" --version" "$RepoRoot" "Smoke test failed"
    ok
}
if ($NoBuild) {
    info "Build skipped (--NoBuild). Using existing binaries."
    if (-not (Test-Path "$OutputDir\photopipeline.exe")) {
        warn "Binary not found at $OutputDir\photopipeline.exe — run without --NoBuild first"
    }
}

# ── Level 1: Unit tests ──────────────────────────────────────
if ($Level1 -or $Level2 -or $Level3 -or $Level4) {
    header "[L1] Unit tests"
    $unitPackages = "photopipeline-core,photopipeline-plugin,photopipeline-engine,photopipeline-plugins,photopipeline-server"
    cargo "test -p $($unitPackages -replace ',',' -p ') --no-fail-fast --features `"$Features`""
    ok
}

# ── Level 2: Integration tests ───────────────────────────────
if ($Level2 -or $Level3 -or $Level4) {
    header "[L2] Integration tests"
    $integPackages = "test-harness"
    cargo "test -p $integPackages --features `"$Features`"" 2>&1 | Out-Null
    # Run e2e crate (cli_system generates binary)
    $e2eCrate = "photopipeline-e2e-tests"
    cargo "test -p $e2eCrate --no-fail-fast --features `"$Features`""
    ok

    header "[L2] gRPC E2E tests"
    $grpcCrate = "photopipeline-grpc-e2e-tests"
    cargo "test -p $grpcCrate --no-fail-fast --features `"$Features`""
    ok

    header "[L2] Pipeline integration tests"
    $pipeCrate = "pipeline-integration-tests"
    cargo "test -p $pipeCrate --no-fail-fast --features `"$Features`""
    ok
}

# ── Level 3: E2E suite (packaged binary required) ────────────
if ($Level3 -or $Level4) {
    if (-not $NoE2E) {
        $e2eRunner = "$OutputDir\photopipeline-e2e-runner.exe"
        $binary = "$OutputDir\photopipeline.exe"

        if (-not (Test-Path $e2eRunner)) {
            fatal "E2E runner not found: $e2eRunner (build first)"
        }
        if (-not (Test-Path $binary)) {
            fatal "Binary not found: $binary (build + stage first)"
        }

        # Enforce DLL requirements for E2E
        $requiredE2E = @("heif.dll", "jxl.dll", "raw.dll")
        foreach ($dll in $requiredE2E) {
            if (-not (Test-Path "$OutputDir\$dll")) {
                fatal "E2E requires $dll in $OutputDir. Build with --Features or install via vcpkg."
            }
        }

        header "[L3] E2E Suite: 520 tests, serial, packaged binary"
        info "Binary: $binary"
        info "Runner: $e2eRunner"
        $reportPath = "$RepoRoot\e2e_report.json"
        $outputPath = "$RepoRoot\tests\e2e_suite\output"

        $e2eCmd = "`"$e2eRunner`" --binary `"$binary`" --categories all --output-dir `"$outputPath`" --report `"$reportPath`""
        info "Running: $e2eCmd"
        cmd /c "$e2eCmd 2>&1"

        if ($LASTEXITCODE -ne 0) {
            warn "E2E suite completed with exit code $LASTEXITCODE (see $reportPath)"
        } else {
            ok
        }

        if (Test-Path $reportPath) {
            info "Report: $reportPath"
        }
    } else {
        info "E2E skipped (--NoE2E)"
    }
}

# ── Level 4: Stress loops ────────────────────────────────────
if ($Level4) {
    header "[L4] Stress: 5 iterations"
    for ($i = 1; $i -le 5; $i++) {
        Write-Host "    === Iteration $i/5 ===" -ForegroundColor Cyan
        cargo "test --workspace --no-fail-fast --features `"$Features`" -- --test-threads=1"
        Write-Host "    === Iteration $i OK ===" -ForegroundColor DarkGray
    }
    ok
}

# ── Done ──────────────────────────────────────────────────────
Write-Host "============================================" -ForegroundColor Green
Write-Host " CI PASSED" -ForegroundColor Green
Write-Host " Package: $OutputDir\" -ForegroundColor Green
Write-Host " Binary : $OutputDir\photopipeline.exe" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
