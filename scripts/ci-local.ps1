param(
    [switch]$Level1,
    [switch]$Level2,
    [switch]$Level3,
    [switch]$Level4,
    [switch]$Clean,
    [switch]$NoBuild,
    [switch]$NoE2E,
    [string]$Features = "libheif-native,libjxl-native,libraw-native",
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Continue"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path "$ScriptDir\.."
if (-not $OutputDir) { $OutputDir = "$RepoRoot\staging" }

# Logging
$ts      = Get-Date -Format "yyyyMMdd_HHmmss"
$LogDir  = "$RepoRoot\ci-logs\$ts"
$MainLog = "$LogDir\ci-main.log"
$StepDir = "$LogDir\steps"
$RootLog = "$RepoRoot\ci-output.log"
$Timing  = @{}
New-Item -ItemType Directory -Force -Path $LogDir, $StepDir | Out-Null
Remove-Item $RootLog -Force -ErrorAction SilentlyContinue

function wlog($m, $c) {
    $line = "[{0}] {1}" -f (Get-Date -Format "HH:mm:ss.fff"), $m
    Write-Host $line -ForegroundColor $c
    Add-Content -Path $MainLog -Value $line
    Add-Content -Path $RootLog -Value $line
}
function header($t) { wlog "" White; wlog "=== $t ===" Yellow; wlog "" White }
function ok()         { wlog "  -> OK" Green }
function fatal($t)   { wlog "  -> FAIL: $t" Red; exit 1 }
function warn($t)    { wlog "  -> WARN: $t" Magenta }
function info($t)    { wlog "  $t" Gray }

function timer_start($n) {
    $Timing[$n] = @{ Start = Get-Date; End = $null; ExitCode = $null }
}
function timer_end($n, $code) {
    $Timing[$n].End      = Get-Date
    $Timing[$n].ExitCode = $code
    $elapsed = "{0:F1}s" -f ($Timing[$n].End - $Timing[$n].Start).TotalSeconds
    wlog "  [$n] $elapsed  exit=$code" Gray
}

function run_step($name, $cmd) {
    timer_start $name
    $slog = "$StepDir\$name.log"
    info "Log: $slog"
    $escaped = "& { $cmd } 2>&1"
    $sb = [ScriptBlock]::Create($escaped)
    try {
        & $sb | ForEach-Object {
            $line = $_.ToString()
            Add-Content -Path $slog -Value $line
            Write-Host $line
        }
        $ec = $LASTEXITCODE
        if ($ec -eq $null) { $ec = 0 }
    } catch {
        $ec = 1
        Add-Content -Path $slog -Value $_.Exception.Message
        Write-Host $_.Exception.Message -ForegroundColor Red
    }
    timer_end $name $ec
    return $ec
}

# Env
$env:VCPKG_ROOT = if (Test-Path "C:\vcpkg") { "C:\vcpkg" } else { $env:VCPKG_ROOT }
# libheif-rs: use x64-windows dynamic triplet (heif.dll) not static-md
$env:VCPKGRS_TRIPLET = "x64-windows"
$env:VCPKGRS_DYNAMIC = "1"
if (-not $env:CARGO_HOME) { $env:CARGO_HOME = "$env:USERPROFILE\.cargo" }
$env:PATH = "$env:CARGO_HOME\bin;$env:PATH"

$protocPath = "C:\Users\GMW\AppData\Local\Microsoft\WinGet\Packages\Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe\bin\protoc.exe"
if (Test-Path $protocPath) { $env:PROTOC = $protocPath }

if (-not ($Level1 -or $Level2 -or $Level3 -or $Level4)) { $Level3 = $true }

$lbl = if ($Level1) { "1 - Unit only" }
  elseif ($Level2) { "2 - Unit+Integration" }
  elseif ($Level3) { "3 - Full CI + E2E 520" }
  elseif ($Level4) { "4 - Full + Stress x5" }

wlog "============================================" Cyan
wlog " Photopipeline CI (PowerShell)" Cyan
wlog " Level   : $lbl" Cyan
wlog " Features: $Features" Cyan
wlog " Output  : $OutputDir" Cyan
wlog " LogDir  : $LogDir" Cyan
wlog "============================================" Cyan

if ($Clean) {
    header "[Clean]"
    Remove-Item "$RepoRoot\staging" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item "$RepoRoot\dist" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item "$RepoRoot\target\ci" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item "$RepoRoot\target\debug\deps\*.dll" -Force -ErrorAction SilentlyContinue
    Remove-Item "$RepoRoot\e2e_report.json" -Force -ErrorAction SilentlyContinue
    ok
}

$needBuild = ($Level3 -or $Level4) -and -not $NoBuild
if ($needBuild) {
    header "[Build] main"
    $rc = run_step "build_main" "cargo build --features $Features"
    if ($rc -ne 0) { fatal "main build (exit $rc)" }
    header "[Build] e2e-runner"
    $rc = run_step "build_runner" "cargo build -p photopipeline-e2e-runner"
    if ($rc -ne 0) { fatal "e2e runner build (exit $rc)" }
    ok
}

header "[Stage] DLLs"
$VcpkgBin = "$env:VCPKG_ROOT\installed\x64-windows\bin"
if (-not (Test-Path $VcpkgBin)) { fatal "VCPKG: $VcpkgBin" }
Copy-Item "$VcpkgBin\*.dll" "$RepoRoot\target\debug\" -Force -ErrorAction SilentlyContinue
Copy-Item "$VcpkgBin\*.dll" "$RepoRoot\target\debug\deps\" -Force -ErrorAction SilentlyContinue
foreach ($dll in @("heif.dll","jxl.dll","raw.dll","raw_r.dll")) {
    $p = "$RepoRoot\target\debug\deps\$dll"
    if (Test-Path $p) {
        $sz = [math]::Round((Get-Item $p).Length / 1024.0, 1)
        info "$dll  ${sz}KB"
    } else {
        fatal "DLL missing: $dll"
    }
}
ok

header "[Stage] Package"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
Copy-Item "$RepoRoot\target\debug\photopipeline.exe" $OutputDir -Force
Copy-Item "$VcpkgBin\*.dll" $OutputDir -Force
info "Staged to $OutputDir"
header "[Smoke]"
$rc = run_step "smoke" "$OutputDir\photopipeline.exe --version"
if ($rc -ne 0) { warn "smoke (exit $rc)" } else { ok }

header "[L1] Unit"
$rc = run_step "L1_unit" "cargo test -p photopipeline-core -p photopipeline-plugin -p photopipeline-engine -p photopipeline-plugins -p photopipeline-server -p test-harness --no-fail-fast --features $Features"
if ($rc -ne 0) { warn "L1 failures (exit $rc)" } else { ok }

if ($Level2 -or $Level3 -or $Level4) {
    header "[L2] gRPC"
    $rc = run_step "L2_grpc" "cargo test -p photopipeline-grpc-e2e-tests --no-fail-fast"
    if ($rc -ne 0) { warn "gRPC (exit $rc)" } else { ok }
    header "[L2] Pipeline"
    $rc = run_step "L2_pipeline" "cargo test -p pipeline-integration-tests --no-fail-fast"
    if ($rc -ne 0) { warn "pipeline (exit $rc)" } else { ok }
    header "[L2] E2E crate"
    $rc = run_step "L2_e2e_crate" "cargo test -p photopipeline-e2e-tests --no-fail-fast"
    if ($rc -ne 0) { warn "e2e-crate (exit $rc)" } else { ok }
    header "[L2] Stress"
    $rc = run_step "L2_stress" "cargo test -p photopipeline-stress-tests --no-fail-fast"
    if ($rc -ne 0) { warn "stress (exit $rc)" } else { ok }
}

if (($Level3 -or $Level4) -and -not $NoE2E) {
    $Runner = "$OutputDir\photopipeline-e2e-runner.exe"
    $Binary = "$OutputDir\photopipeline.exe"
    Copy-Item "$RepoRoot\target\debug\photopipeline.exe" $OutputDir -Force
    Copy-Item "$RepoRoot\target\debug\photopipeline-e2e-runner.exe" $OutputDir -Force -ErrorAction SilentlyContinue
    if (-not (Test-Path $Runner)) {
        $rc = run_step "build_runner_retry" "cargo build -p photopipeline-e2e-runner"
        if ($rc -ne 0) { fatal "runner build (exit $rc)" }
        Copy-Item "$RepoRoot\target\debug\photopipeline-e2e-runner.exe" $OutputDir -Force
    }
    foreach ($d in @("heif.dll","jxl.dll","raw.dll")) {
        if (-not (Test-Path "$OutputDir\$d")) { fatal "E2E DLL: $d" }
    }
    header "[L3] E2E 520"
    $e2eLog = "$StepDir\L3_e2e.log"
    $e2eRpt = "$RepoRoot\e2e_report.json"
    $e2eOut = "$RepoRoot\tests\e2e_suite\output"
    info "E2E log: $e2eLog"
    timer_start "L3_E2E"
    Push-Location $OutputDir
    try {
        $e2eArgs = @("--binary", ".\photopipeline.exe", "--categories", "all", "--output-dir", $e2eOut, "--report", $e2eRpt)
        & .\photopipeline-e2e-runner.exe @e2eArgs 2>&1 | ForEach-Object {
            $line = $_.ToString()
            Add-Content -Path $e2eLog -Value $line
            Write-Host $line
        }
        $e2eExit = $LASTEXITCODE
        if ($e2eExit -eq $null) { $e2eExit = 0 }
    } finally {
        Pop-Location
    }
    timer_end "L3_E2E" $e2eExit
    if (Test-Path $e2eRpt) {
        try {
            $rpt = Get-Content $e2eRpt -Raw | ConvertFrom-Json
            info "Report: total=$($rpt.total) passed=$($rpt.passed) failed=$($rpt.failed)"
        } catch {
            info "Report: parse failed"
        }
    }
    if ($e2eExit -ne 0) { warn "E2E runner exit=$e2eExit" } else { ok }
}

if ($Level4) {
    header "[L4] Stress x5"
    for ($i = 1; $i -le 5; $i++) {
        Write-Host "    === $i/5 ===" -ForegroundColor Yellow
        $rc = run_step "L4_stress_$i" "cargo test --workspace --no-fail-fast --features $Features -- --test-threads=1"
        if ($rc -ne 0) { warn "stress $i (exit $rc)" }
    }
    ok
}

# Summary
wlog "" White
wlog "============================================" Cyan
wlog " CI Summary" Cyan
wlog "============================================" Cyan
$overall = 0
foreach ($k in $Timing.Keys | Sort-Object) {
    $t   = $Timing[$k]
    $dur = if ($t.End) { "{0:F1}s" -f ($t.End - $t.Start).TotalSeconds } else { "-----" }
    $st  = if ($t.ExitCode -eq 0) { "PASS" }
           elseif ($t.ExitCode -ne $null) { "FAIL" }
           else { "----" }
    $fg  = if ($st -eq "PASS") { "Green" }
           elseif ($st -eq "FAIL") { "Red" }
           else { "Gray" }
    wlog ("  {0,-5}  {1,28}  {2,8}" -f $st, $k, $dur) $fg
    if ($st -eq "FAIL") { $overall = 1 }
}
wlog "" White
$cnt = (Get-ChildItem -Path $StepDir -Filter "*.log" | Measure-Object).Count
wlog " Logs: $LogDir ($cnt step files)" Cyan
wlog " Main: $MainLog" Cyan
wlog " Root: $RootLog" Cyan
if ($overall -eq 0) {
    wlog "============================================" Green
    wlog "  ALL PASSED" Green
    wlog "============================================" Green
} else {
    wlog "============================================" Red
    wlog "  SOME STEPS FAILED" Red
    wlog "============================================" Red
}
exit $overall
