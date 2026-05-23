# Vendor Libraries

This directory contains build scripts for third-party C/C++ libraries
that are bundled with Photopipeline.

## Building

```bash
./vendor/build-all.sh
```

This downloads and compiles all required libraries, installing them to `vendor/install/`.

## Included Libraries

| Library | Version | Purpose |
|---------|---------|---------|
| x265 | 3.6 | HEVC encoding (highest quality) |
| libde265 | 1.0.15 | HEVC decoding |
| libheif | 1.18.2 | HEIF/HEIC container |
| libaom | 3.9 | AV1 encoding (via libheif) |
| libjxl | 0.11.0 | JPEG XL encoding/decoding |
| lcms2 | 2.16 | ICC color management |
| LibRaw | 0.21.3 | RAW camera format decoding |
| libpng | 1.6.43 | PNG encoding/decoding |
| libtiff | 4.6.0 | TIFF encoding/decoding |
| libjpeg-turbo | 3.0 | JPEG encoding/decoding (optimized) |

## Bundling

After building, the `scripts/collect-libs.sh` script copies the necessary
shared libraries into the release package so users don't need to install
anything separately.
