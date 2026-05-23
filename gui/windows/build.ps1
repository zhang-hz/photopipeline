# Builds the WinUI project (used by CI)
param(
    [string]$Configuration = "Release",
    [string]$Architecture = "x64"
)

$env:Path = "C:\Program Files\dotnet;$env:Path"

Write-Host "=== Photopipeline WinUI 3 Build ===" -ForegroundColor Cyan
Write-Host "Configuration: $Configuration"
Write-Host "Architecture: $Architecture"
Write-Host ""

Write-Host "[1/3] Restoring packages..." -ForegroundColor Yellow
& dotnet restore Photopipeline/Photopipeline.csproj
if ($LASTEXITCODE -ne 0) {
    Write-Host "Package restore failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "[2/3] Building project..." -ForegroundColor Yellow
& dotnet build Photopipeline/Photopipeline.csproj -c $Configuration -a $Architecture
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "[3/3] Publishing..." -ForegroundColor Yellow
& dotnet publish Photopipeline/Photopipeline.csproj -c $Configuration -a $Architecture -o ./publish/
if ($LASTEXITCODE -ne 0) {
    Write-Host "Publish failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "Build complete. Output in ./publish/" -ForegroundColor Green
