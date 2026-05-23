#!/bin/bash
set -euo pipefail

BINARY="${1:-target/release/photopipeline}"
OUTDIR="${2:-release/lib}"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Usage: $0 <binary-path> <output-dir>"
    exit 1
fi

mkdir -p "$OUTDIR"

echo "Collecting shared libraries for: $BINARY"
echo "Output directory: $OUTDIR"
echo ""

case "$(uname -s)" in
    Linux)
        echo "Platform: Linux"
        echo "Using ldd to find dependencies..."

        LIBS=$(ldd "$BINARY" 2>/dev/null | grep "=> /" | awk '{print $3}' || true)

        for lib in $LIBS; do
            basename_lib=$(basename "$lib")
            if echo "$basename_lib" | grep -qE "^(libc\.|libm\.|libpthread|libdl\.|libgcc_s|libstdc\+\+|ld-linux|linux-vdso|libselinux|libcap|libcrypt|libresolv)"; then
                continue
            fi
            if [ -f "$lib" ]; then
                cp -v "$lib" "$OUTDIR/"
            fi
        done
        ;;

    Darwin)
        echo "Platform: macOS"
        echo "Using otool to find dependencies..."

        LIBS=$(otool -L "$BINARY" 2>/dev/null | awk '{print $1}' | grep -v "^$BINARY" | grep -v "^/usr/lib/" | grep -v "^/System/Library/" | grep "\.dylib" || true)

        for lib in $LIBS; do
            if [ -f "$lib" ]; then
                cp -v "$lib" "$OUTDIR/"
                basename_lib=$(basename "$lib")
                install_name_tool -id "@rpath/$basename_lib" "$OUTDIR/$basename_lib" 2>/dev/null || true
            fi
        done

        install_name_tool -add_rpath "@executable_path/../lib" "$BINARY" 2>/dev/null || true
        ;;

    MINGW*|MSYS*|CYGWIN*)
        echo "Platform: Windows"
        echo "DLL search path is the executable directory. Copy DLLs next to .exe."
        ;;

    *)
        echo "Unknown platform: $(uname -s)"
        exit 1
        ;;
esac

echo ""
echo "Collected $(ls "$OUTDIR" 2>/dev/null | wc -l) libraries into $OUTDIR"
echo "Done."
