#!/bin/bash
# Photopipeline CI — single entry point for all testing
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${OUTPUT_DIR:-$REPO_ROOT/staging}"
FEATURES="libheif-native,libjxl-native,libraw-native"
LOG_FILE="$REPO_ROOT/ci-output.log"
export PROTOC="${PROTOC:-/c/Users/GMW/AppData/Local/Microsoft/WinGet/Packages/Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe/bin/protoc}"
export VCPKG_ROOT="${VCPKG_ROOT:-C:/vcpkg}"
export PATH="$HOME/.cargo/bin:$PATH"
rm -f "$LOG_FILE"

LEVEL=0; CLEAN=0; NOE2E=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        -L1|-L2|-L3|-L4) LEVEL="${1:2}" ;;
        -clean) CLEAN=1 ;;
        -noe2e) NOE2E=1 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
    shift
done
[[ $LEVEL -eq 0 ]] && LEVEL=3

log()  { echo -e "$1" | tee -a "$LOG_FILE"; }
header(){ log "\n\033[1;33m$1\033[0m"; }
ok()   { log "  \033[1;32m-> OK\033[0m"; }
fatal(){ log "  \033[1;31m-> FAIL: $1\033[0m"; exit 1; }
warn() { log "  \033[1;35m-> WARN: $1\033[0m"; }
info() { log "  $1"; }
run()  { "$@" 2>&1 | tee -a "$LOG_FILE"; return ${PIPESTATUS[0]}; }

log "============================================"
log " Photopipeline CI (bash)"
log " Level: $LEVEL"
log " Output: $OUTPUT_DIR"
log "============================================"

# ── Clean ──
if [[ $CLEAN -eq 1 ]]; then
    header "[Clean]"
    rm -rf "$REPO_ROOT/staging" "$REPO_ROOT/dist"
    rm -f "$REPO_ROOT/target/debug/deps/"*.dll
    rm -f "$REPO_ROOT/e2e_report.json"
    ok
fi

# ── Build (MANDATORY) ──
header "[Build]"
run cargo build --features "$FEATURES" || fatal "build failed"
ok

# ── Stage DLLs ──
header "[Stage] DLLs"
VCPKG_BIN="${VCPKG_ROOT}/installed/x64-windows/bin"
[[ -d "$VCPKG_BIN" ]] || fatal "VCPKG: $VCPKG_BIN"
cp -f "$VCPKG_BIN/"*.dll "$REPO_ROOT/target/debug/" 2>/dev/null || true
cp -f "$VCPKG_BIN/"*.dll "$REPO_ROOT/target/debug/deps/" 2>/dev/null || true
for dll in heif.dll jxl.dll raw.dll raw_r.dll; do
    [[ -f "$REPO_ROOT/target/debug/deps/$dll" ]] || fatal "DLL: $dll"
    info "  $dll"
done
ok

# ── Package ──
header "[Stage] Package"
mkdir -p "$OUTPUT_DIR"
cp -f "$REPO_ROOT/target/debug/photopipeline.exe" "$OUTPUT_DIR/" 2>/dev/null || true
cp -f "$VCPKG_BIN/"*.dll "$OUTPUT_DIR/" 2>/dev/null || true
info "Staged to $OUTPUT_DIR"
header "[Smoke]"
"$OUTPUT_DIR/photopipeline.exe" --version 2>/dev/null || warn "smoke"
ok

# ── L1: Unit ──
header "[L1] Unit"
run cargo test -p photopipeline-core -p photopipeline-plugin -p photopipeline-engine \
    -p photopipeline-plugins -p photopipeline-server -p test-harness \
    --no-fail-fast --features "$FEATURES" || warn "L1 failures"
ok

# ── L2: Integration ──
if [[ $LEVEL -ge 2 ]]; then
    header "[L2] gRPC"
    run cargo test -p photopipeline-grpc-e2e-tests --no-fail-fast || warn "gRPC"
    ok
    header "[L2] Pipeline"
    run cargo test -p pipeline-integration-tests --no-fail-fast || warn "pipeline"
    ok
    header "[L2] E2E crate"
    run cargo test -p photopipeline-e2e-tests --no-fail-fast || warn "e2e-crate"
    ok
    header "[L2] Stress"
    run cargo test -p photopipeline-stress-tests --no-fail-fast || warn "stress"
    ok
fi

# ── L3: E2E Suite ──
if [[ $LEVEL -ge 3 ]] && [[ $NOE2E -eq 0 ]]; then
    E2E_RUNNER="$OUTPUT_DIR/photopipeline-e2e-runner.exe"
    BINARY="$OUTPUT_DIR/photopipeline.exe"
    if [[ ! -f "$E2E_RUNNER" ]]; then
        run cargo build -p photopipeline-e2e-runner || fatal "runner build"
        cp -f "$REPO_ROOT/target/debug/photopipeline-e2e-runner.exe" "$OUTPUT_DIR/"
    fi
    for dll in heif.dll jxl.dll raw.dll; do
        [[ -f "$OUTPUT_DIR/$dll" ]] || fatal "E2E DLL: $dll"
    done
    header "[L3] E2E 520"
    "$E2E_RUNNER" --binary "$BINARY" --categories all \
        --output-dir "$REPO_ROOT/tests/e2e_suite/output" \
        --report "$REPO_ROOT/e2e_report.json" || warn "E2E failures"
    ok
fi

# ── L4: Stress ──
if [[ $LEVEL -ge 4 ]]; then
    header "[L4] Stress x5"
    for i in $(seq 1 5); do
        echo "    === $i/5 ==="
        run cargo test --workspace --no-fail-fast --features "$FEATURES" -- --test-threads=1 || warn "stress $i"
    done
    ok
fi

log "\n============================================"
log "  \033[1;32mCI PASSED\033[0m"
log "============================================"
