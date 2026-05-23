#!/bin/bash
set -euo pipefail

VENDOR_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_DIR="${VENDOR_DIR}/install"
NPROC=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

mkdir -p "${INSTALL_DIR}/lib" "${INSTALL_DIR}/include"
export PKG_CONFIG_PATH="${INSTALL_DIR}/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
export LD_LIBRARY_PATH="${INSTALL_DIR}/lib:${LD_LIBRARY_PATH:-}"

echo "=== Photopipeline Vendor Build ==="
echo "Install directory: ${INSTALL_DIR}"
echo "Parallel jobs: ${NPROC}"
echo ""

# ---- Build Functions ----

build_x265() {
    echo "--- Building x265 v3.6 ---"
    local SRC="${VENDOR_DIR}/x265_3.6"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading x265 v3.6..."
        curl -fsSL "https://bitbucket.org/multicoreware/x265_git/downloads/x265_3.6.tar.gz" | tar xz -C "${VENDOR_DIR}"
        mv "${VENDOR_DIR}/x265_3.6" "${SRC}" 2>/dev/null || true
    fi
    if [ -f "${INSTALL_DIR}/lib/libx265.so" ]; then
        echo "x265 already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build_10bit"
    cd "${SRC}/build_10bit"
    cmake ../source \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DHIGH_BIT_DEPTH=ON \
        -DMAIN12=OFF \
        -DENABLE_SHARED=ON \
        -DENABLE_CLI=OFF \
        -DEXPORT_C_API=ON
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "x265 v3.6 built"
}

build_libde265() {
    echo "--- Building libde265 v1.0.15 (HEVC decoder) ---"
    local SRC="${VENDOR_DIR}/libde265-1.0.15"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading libde265 v1.0.15..."
        curl -fsSL "https://github.com/strukturag/libde265/releases/download/v1.0.15/libde265-1.0.15.tar.gz" | tar xz -C "${VENDOR_DIR}"
    fi
    if [ -f "${INSTALL_DIR}/lib/libde265.so" ]; then
        echo "libde265 already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -DENABLE_DECODER=ON \
        -DENABLE_ENCODER=OFF
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "libde265 v1.0.15 built"
}

build_libheif() {
    echo "--- Building libheif v1.18.2 ---"
    local SRC="${VENDOR_DIR}/libheif-1.18.2"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading libheif v1.18.2..."
        curl -fsSL "https://github.com/strukturag/libheif/archive/refs/tags/v1.18.2.tar.gz" | tar xz -C "${VENDOR_DIR}"
    fi
    if [ -f "${INSTALL_DIR}/lib/libheif.so" ]; then
        echo "libheif already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DCMAKE_PREFIX_PATH="${INSTALL_DIR}" \
        -DWITH_X265=ON \
        -DWITH_LIBDE265=ON \
        -DWITH_AOM_DECODER=OFF \
        -DWITH_DAV1D=OFF \
        -DWITH_RAV1E=OFF \
        -DWITH_SvtEnc=OFF \
        -DENABLE_PLUGIN_LOADING=OFF \
        -DBUILD_SHARED_LIBS=ON \
        -DCMAKE_POSITION_INDEPENDENT_CODE=ON
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "libheif v1.18.2 built"
}

build_libaom() {
    echo "--- Building libaom v3.9 (AV1 codec) ---"
    local SRC="${VENDOR_DIR}/libaom-3.9"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading libaom v3.9..."
        curl -fsSL "https://github.com/AOMediaCodec/libaom/archive/refs/tags/v3.9.0.tar.gz" | tar xz -C "${VENDOR_DIR}"
        mv "${VENDOR_DIR}/libaom-3.9.0" "${SRC}" 2>/dev/null || true
    fi
    if [ -f "${INSTALL_DIR}/lib/libaom.so" ]; then
        echo "libaom already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -DENABLE_DOCS=OFF \
        -DENABLE_TESTS=OFF \
        -DENABLE_TOOLS=OFF \
        -DENABLE_EXAMPLES=OFF
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "libaom v3.9 built"
}

build_libjxl() {
    echo "--- Building libjxl v0.11.0 ---"
    local SRC="${VENDOR_DIR}/libjxl-0.11.0"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading libjxl v0.11.0..."
        curl -fsSL "https://github.com/libjxl/libjxl/archive/refs/tags/v0.11.0.tar.gz" | tar xz -C "${VENDOR_DIR}"
    fi
    if [ -f "${INSTALL_DIR}/lib/libjxl.so" ]; then
        echo "libjxl already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -DJPEGXL_ENABLE_TOOLS=OFF \
        -DJPEGXL_ENABLE_DOXYGEN=OFF \
        -DJPEGXL_ENABLE_MANPAGES=OFF \
        -DJPEGXL_ENABLE_JNI=OFF \
        -DJPEGXL_ENABLE_SJPEG=OFF \
        -DJPEGXL_ENABLE_OPENEXR=OFF \
        -DJPEGXL_ENABLE_SKCMS=OFF \
        -DJPEGXL_FORCE_SYSTEM_HWY=OFF \
        -DJPEGXL_FORCE_SYSTEM_BROTLI=OFF
    cmake --build . --parallel "${NPROC}" --target jxl jxl_threads
    # Manual install of libjxl (cmake --install may not work for sub-targets)
    cp lib/libjxl.so* "${INSTALL_DIR}/lib/"
    cp lib/libjxl_threads.so* "${INSTALL_DIR}/lib/"
    mkdir -p "${INSTALL_DIR}/include/jxl"
    cp ../lib/include/jxl/*.h "${INSTALL_DIR}/include/jxl/"
    echo "libjxl v0.11.0 built"
}

build_lcms2() {
    echo "--- Building lcms2 v2.16 ---"
    local SRC="${VENDOR_DIR}/lcms2-2.16"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading lcms2 v2.16..."
        curl -fsSL "https://github.com/mm2/Little-CMS/archive/refs/tags/lcms2.16.tar.gz" | tar xz -C "${VENDOR_DIR}"
    fi
    if [ -f "${INSTALL_DIR}/lib/liblcms2.so" ]; then
        echo "lcms2 already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -DBUILD_TESTS=OFF \
        -DBUILD_UTILS=OFF
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "lcms2 v2.16 built"
}

build_libraw() {
    echo "--- Building LibRaw v0.21.3 ---"
    local SRC="${VENDOR_DIR}/LibRaw-0.21.3"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading LibRaw v0.21.3..."
        curl -fsSL "https://github.com/LibRaw/LibRaw/archive/refs/tags/0.21.3.tar.gz" | tar xz -C "${VENDOR_DIR}"
    fi
    if [ -f "${INSTALL_DIR}/lib/libraw.so" ]; then
        echo "LibRaw already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -DBUILD_TOOLS=OFF \
        -DENABLE_JASPER=OFF \
        -DENABLE_LCMS=OFF \
        -DENABLE_RAWSPEED=ON
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "LibRaw v0.21.3 built"
}

build_libpng() {
    echo "--- Building libpng v1.6.43 ---"
    local SRC="${VENDOR_DIR}/libpng-1.6.43"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading libpng v1.6.43..."
        curl -fsSL "https://download.sourceforge.net/libpng/libpng-1.6.43.tar.xz" | tar xJ -C "${VENDOR_DIR}"
    fi
    if [ -f "${INSTALL_DIR}/lib/libpng16.so" ]; then
        echo "libpng already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -DPNG_TESTS=OFF \
        -DPNG_TOOLS=OFF
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "libpng v1.6.43 built"
}

build_libtiff() {
    echo "--- Building libtiff v4.6.0 ---"
    local SRC="${VENDOR_DIR}/libtiff-4.6.0"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading libtiff v4.6.0..."
        curl -fsSL "https://download.osgeo.org/libtiff/tiff-4.6.0.tar.gz" | tar xz -C "${VENDOR_DIR}"
    fi
    if [ -f "${INSTALL_DIR}/lib/libtiff.so" ]; then
        echo "libtiff already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -Dtiff-tools=OFF \
        -Dtiff-tests=OFF \
        -Dtiff-docs=OFF
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "libtiff v4.6.0 built"
}

build_libjpeg_turbo() {
    echo "--- Building libjpeg-turbo v3.0 ---"
    local SRC="${VENDOR_DIR}/libjpeg-turbo-3.0"
    if [ ! -d "${SRC}" ]; then
        echo "Downloading libjpeg-turbo v3.0..."
        curl -fsSL "https://github.com/libjpeg-turbo/libjpeg-turbo/archive/refs/tags/3.0.0.tar.gz" | tar xz -C "${VENDOR_DIR}"
        mv "${VENDOR_DIR}/libjpeg-turbo-3.0.0" "${SRC}" 2>/dev/null || true
    fi
    if [ -f "${INSTALL_DIR}/lib/libjpeg.so" ]; then
        echo "libjpeg-turbo already built, skipping"
        return
    fi
    mkdir -p "${SRC}/build"
    cd "${SRC}/build"
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}" \
        -DBUILD_SHARED_LIBS=ON \
        -DENABLE_STATIC=OFF
    cmake --build . --parallel "${NPROC}"
    cmake --install .
    echo "libjpeg-turbo v3.0 built"
}

# ---- Main ----
echo "Starting builds at $(date)"
echo ""

build_libde265
build_libpng
build_libjpeg_turbo
build_libtiff
build_x265
build_libheif
build_lcms2
build_libraw
build_libjxl
build_libaom

echo ""
echo "=== All vendor libraries built ==="
echo "Install prefix: ${INSTALL_DIR}"
echo "Libraries:"
ls -la "${INSTALL_DIR}/lib/"*.so* 2>/dev/null || echo "  (no .so files found)"
echo ""
echo "Done at $(date)"
