set export := true
export VCPKG_ROOT := env_var_or("VCPKG_ROOT", "C:/vcpkg")
VCPKG_BIN := VCPKG_ROOT / "installed" / "x64-windows" / "bin"

# === 编译 ===
@build:
    cargo build --workspace

@build-release:
    cargo build --profile ci --workspace

@gui-build:
    dotnet build gui/windows/Photopipeline.sln -c Release -p:Platform=x64

# === 测试 ===
@test:
    cargo test --workspace --no-fail-fast

@gui-test:
    dotnet test gui/windows/Photopipeline.Tests/Photopipeline.Tests.csproj -c Release -p:Platform=x64 --no-build --filter "FullyQualifiedName~UnitTests"

# === 代码质量 ===
@lint:
    cargo clippy --workspace

@fmt:
    cargo fmt --all

@fmt-check:
    cargo fmt --all -- --check

@security:
    cargo audit

# === CI 模拟 ===
@ci-build:
    just build-release
    just gui-build

@ci-test:
    just test
    just gui-test

@ci-lint:
    just lint
    just fmt-check

@ci-all: ci-build ci-test ci-lint

# === 发布打包 ===
@gui-publish-x64:
    dotnet publish gui/windows/Photopipeline/Photopipeline.csproj -c Release -p:Platform=x64 -r win-x64 -o publish/win-x64 --self-contained -p:WindowsAppSDKSelfContained=true

@stage-x64:
    powershell -File scripts/stage.ps1

@installer-x64:
    @echo "==> Building installer with Inno Setup..."
    powershell -NoProfile -Command "$$iscc = @('C:\Program Files (x86)\Inno Setup 6\ISCC.exe', 'C:\Users\GMW\inno-setup\ISCC.exe') | Where-Object { Test-Path $$_ } | Select-Object -First 1; if (-not $$iscc) { Write-Error 'Inno Setup not found. Install: choco install innosetup -y'; exit 1 }; Write-Host \"Using: $$iscc\"; & $$iscc scripts/installer/photopipeline.iss"

@zip-x64:
    powershell -NoProfile -Command "Compress-Archive -Path staging\* -DestinationPath dist/photopipeline-windows-x64-portable.zip -Force"

@sign-x64:
    @echo "==> Signing EXE and DLL in staging/..."
    powershell -NoProfile -Command "Get-ChildItem -Recurse -Include '*.exe','*.dll' -Path 'staging' | ForEach-Object { signtool sign /fd SHA256 /f photopipeline-codesign.pfx /p $env:SIGNING_CERTIFICATE_PASSWORD /tr http://timestamp.digicert.com /td SHA256 /v $_.FullName }"

@release-full-x64: build-release gui-publish-x64 stage-x64 installer-x64 zip-x64
    @echo "==> Done: dist/photopipeline-*-windows-x64-portable.zip"
    @echo "==> Done: dist/photopipeline-*-windows-x64-setup.exe"

@clean-staging:
    @if exist staging rmdir /s /q staging
    @if exist publish rmdir /s /q publish
    @if exist dist  rmdir /s /q dist

# === 清理 ===
@clean:
    cargo clean
