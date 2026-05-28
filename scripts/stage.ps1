param([string]$VcpkgRoot = $env:VCPKG_ROOT, [string]$OutputDir = "staging")

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Unified Rust binary
Copy-Item "target/ci/photopipeline.exe" "$OutputDir/" -ErrorAction Stop

# vcpkg native DLLs
$vcpkgBin = Join-Path $VcpkgRoot "installed/x64-windows/bin"
if (Test-Path $vcpkgBin) {
    Copy-Item "$vcpkgBin/*.dll" "$OutputDir/" -ErrorAction SilentlyContinue
}

# License
if (Test-Path "LICENSE") { Copy-Item "LICENSE" "$OutputDir/license.txt" }

# Verify critical C FFI DLLs
$required = @("heif.dll", "jxl.dll", "raw.dll", "raw_r.dll", "lcms2-2.dll")
foreach ($dll in $required) {
    if (-not (Test-Path "$OutputDir/$dll")) {
        Write-Warning "Missing DLL: $dll — install via vcpkg install <package>"
    }
}

Write-Host "Staged $(Get-ChildItem $OutputDir | Measure-Object | Select-Object -ExpandProperty Count) files to $OutputDir"
