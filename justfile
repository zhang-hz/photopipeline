set export := true
export VCPKG_ROOT := env_var_or("VCPKG_ROOT", "C:/vcpkg")
VCPKG_BIN := VCPKG_ROOT / "installed" / "x64-windows" / "bin"

# === CI (primary entry point) ===
@ci-level1:
    powershell -File scripts/ci-local.ps1 -Level1

@ci-level2:
    powershell -File scripts/ci-local.ps1 -Level2

@ci-level3:
    powershell -File scripts/ci-local.ps1 -Level3

@ci-level4:
    powershell -File scripts/ci-local.ps1 -Level4

@ci-all: ci-level3

# === Build ===
@build:
    cargo build --workspace

@build-release:
    cargo build --profile ci --workspace

# === Code Quality ===
@lint:
    cargo clippy --workspace

@fmt:
    cargo fmt --all

@fmt-check:
    cargo fmt --all -- --check

@security:
    cargo audit

# === Stage ===
@stage:
    powershell -File scripts/stage.ps1

# === Release Packaging ===
@zip:
    powershell -NoProfile -Command "Compress-Archive -Path staging\* -DestinationPath dist/photopipeline-windows-x64-portable.zip -Force"

@release: build-release stage zip
    @echo "==> Done: dist/photopipeline-*-windows-x64-portable.zip"

# === Clean ===
@clean:
    cargo clean

@clean-staging:
    @if exist staging rmdir /s /q staging
    @if exist dist rmdir /s /q dist
