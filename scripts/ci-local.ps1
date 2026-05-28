param(
    [switch]$SkipRust,
    [switch]$SkipTests,
    [string]$OutputDir = ""
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path "$ScriptDir\.."

if (-not $OutputDir) {
    $OutputDir = "$RepoRoot\publish\win-x64"
}

Write-Host "============================================" -ForegroundColor Cyan
Write-Host " Photopipeline Local CI" -ForegroundColor Cyan
Write-Host " Repo  : $RepoRoot" -ForegroundColor Cyan
Write-Host " Output: $OutputDir" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

$env:VCPKG_ROOT = "C:\vcpkg"
$env:VCPKG_DEFAULT_TRIPLET = "x64-windows"

function header([string]$text) { Write-Host $text -ForegroundColor Yellow }
function ok() { Write-Host "  -> OK" -ForegroundColor Green; Write-Host "" }
function fatal([string]$msg) { Write-Host "  -> FAIL: $msg" -ForegroundColor Red; exit 1 }

function sh([string]$cmd, [string]$cwd, [string]$err) {
    $full = if ($cwd) { "cd /d `"$cwd`" && $cmd 2>&1" } else { "$cmd 2>&1" }
    cmd /c $full
    if ($LASTEXITCODE -ne 0) { fatal $err }
}

# ── Rust ────────────────────────────────────────────────────
if (-not $SkipRust) {
    header "[1/6] Rust: cargo build --profile ci --workspace"
    sh "cargo build --profile ci --workspace" "$RepoRoot" "Rust build failed"
    ok

    if (-not $SkipTests) {
        header "[2/6] Rust: cargo test --workspace --no-fail-fast"
        sh "cargo test --workspace --no-fail-fast" "$RepoRoot" "Rust tests failed"
        ok
    } else {
        Write-Host "[2/6] Rust tests: SKIPPED" -ForegroundColor DarkGray
    }

    header "[3/6] Rust: cargo fmt --all -- --check"
    sh "cargo fmt --all -- --check" "$RepoRoot" "Rust fmt check failed"
    ok

    header "[4/6] Rust: cargo clippy --workspace"
    sh "cargo clippy --workspace" "$RepoRoot" "Rust clippy failed"
    ok
} else {
    Write-Host "[1/6] - [4/6] Rust: SKIPPED" -ForegroundColor DarkGray
    Write-Host ""
}

# ── Stage ───────────────────────────────────────────────────
header "[5/6] Stage: scripts/stage.ps1 -> staging/"
sh "powershell -ExecutionPolicy Bypass -File `"$RepoRoot\scripts\stage.ps1`"" "$RepoRoot" "Stage failed"
ok

# ── Smoke ───────────────────────────────────────────────────
header "[6/6] Smoke: staging/photopipeline.exe --version"
$cli = "$RepoRoot\staging\photopipeline.exe"
if (-not (Test-Path $cli)) { fatal "Binary not found at $cli" }
sh "`"$cli`" --version" "$RepoRoot" "Smoke test failed"
ok

Write-Host "============================================" -ForegroundColor Green
Write-Host " CI PASSED" -ForegroundColor Green
Write-Host " Package: $RepoRoot\staging\" -ForegroundColor Green
Write-Host " Launch : $RepoRoot\staging\photopipeline.exe" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
