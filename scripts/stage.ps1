param([string]$VcpkgRoot = $env:VCPKG_ROOT, [string]$OutputDir = "staging")

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Rust binaries (CLI + gRPC server)
Copy-Item "target/ci/photopipeline-cli.exe" "$OutputDir/" -ErrorAction Stop
Copy-Item "target/ci/photopipeline-server.exe" "$OutputDir/" -ErrorAction Stop

# vcpkg native DLLs
$vcpkgBin = Join-Path $VcpkgRoot "installed/x64-windows/bin"
if (Test-Path $vcpkgBin) {
    Copy-Item "$vcpkgBin/*.dll" "$OutputDir/" -ErrorAction SilentlyContinue
}

# GUI self-contained publish
Copy-Item "publish/win-x64/*" "$OutputDir/" -Recurse -Force

# License
if (Test-Path "LICENSE") { Copy-Item "LICENSE" "$OutputDir/license.txt" }

Write-Host "Staged $(Get-ChildItem $OutputDir | Measure-Object | Select-Object -ExpandProperty Count) files to $OutputDir"
