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
    header "[1/9] Rust: cargo build --profile ci --workspace"
    sh "cargo build --profile ci --workspace" "$RepoRoot" "Rust build failed"
    ok

    if (-not $SkipTests) {
        header "[2/9] Rust: cargo test --workspace --no-fail-fast"
        sh "cargo test --workspace --no-fail-fast" "$RepoRoot" "Rust tests failed"
        ok
    } else {
        Write-Host "[2/9] Rust tests: SKIPPED" -ForegroundColor DarkGray
    }

    header "[3/9] Rust: cargo fmt --all -- --check"
    sh "cargo fmt --all -- --check" "$RepoRoot" "Rust fmt check failed"
    ok

    header "[4/9] Rust: cargo clippy --workspace"
    sh "cargo clippy --workspace" "$RepoRoot" "Rust clippy failed"
    ok
} else {
    Write-Host "[1/9] - [4/9] Rust: SKIPPED" -ForegroundColor DarkGray
    Write-Host ""
}
# ─────────────────────────────────────────────────────────────

# ── C# ──────────────────────────────────────────────────────
$SlnDir = "$RepoRoot\gui\windows"

header "[5/9] C#: dotnet build Photopipeline.sln -c Release -p:Platform=x64"
sh "dotnet build Photopipeline.sln -c Release -p:Platform=x64" "$SlnDir" "C# build failed"
ok

if (-not $SkipTests) {
    header "[6/9] C#: dotnet test (L1 + L2) -c Release -p:Platform=x64"
    sh "dotnet test Photopipeline.Tests/Photopipeline.Tests.csproj -c Release -p:Platform=x64 --no-build" "$SlnDir" "C# tests failed"
    ok
} else {
    Write-Host "[6/9] C# tests: SKIPPED" -ForegroundColor DarkGray
    Write-Host ""
}
# ─────────────────────────────────────────────────────────────

# ── GUI publish ─────────────────────────────────────────────
header "[7/9] GUI: dotnet publish -> $OutputDir"
$publishArgs = "publish Photopipeline/Photopipeline.csproj -c Release -p:Platform=x64 -r win-x64 -o `"$OutputDir`" --self-contained "
sh "dotnet $publishArgs" "$SlnDir" "GUI publish failed"
ok
# ─────────────────────────────────────────────────────────────

# ── Stage ───────────────────────────────────────────────────
header "[8/9] Stage: scripts/stage.ps1 -> staging/"
sh "powershell -ExecutionPolicy Bypass -File `"$RepoRoot\scripts\stage.ps1`"" "$RepoRoot" "Stage failed"
ok
# ─────────────────────────────────────────────────────────────

# ── Smoke ───────────────────────────────────────────────────
header "[9/9] Smoke: staging/photopipeline-cli.exe --version"
$cli = "$RepoRoot\staging\photopipeline-cli.exe"
if (-not (Test-Path $cli)) { fatal "CLI binary not found at $cli" }
sh "`"$cli`" --version" "$RepoRoot" "CLI smoke test failed"
ok
# ─────────────────────────────────────────────────────────────

Write-Host "============================================" -ForegroundColor Green
Write-Host " CI PASSED" -ForegroundColor Green
Write-Host " Package: $RepoRoot\staging\" -ForegroundColor Green
Write-Host " Launch : $RepoRoot\staging\Photopipeline.exe" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
