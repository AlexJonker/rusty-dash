#!/bin/bash
set -e

echo "Building aasdk and openauto..."
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AASDK_DIR="$ROOT_DIR/aasdk"
OPENAUTO_DIR="$ROOT_DIR/openauto"
OUTPUT_DIR="${TOP_DIR:-$ROOT_DIR}/output"

AASDK_BUILD_DIR="$OUTPUT_DIR/aasdk"
OPENAUTO_BUILD_DIR="$OUTPUT_DIR/openauto"

mkdir -p "$OUTPUT_DIR" "$OPENAUTO_BUILD_DIR"

cmake -S "$AASDK_DIR" -B "$AASDK_BUILD_DIR"
cmake --build "$AASDK_BUILD_DIR" -j"$(nproc)"

# As seen in `ldd output/bin/autoapp | grep aasdk` these flags should be changed when used in building for the rpi image.
# Should probably end up in something like /opt/nowa
cmake -S "$OPENAUTO_DIR" -B "$OPENAUTO_BUILD_DIR" \
  -DCMAKE_EXE_LINKER_FLAGS="-Wl,--copy-dt-needed-entries" \
  -DAASDK_INCLUDE_DIRS="$AASDK_DIR/include" \
  -DAASDK_PROTO_INCLUDE_DIRS="$AASDK_BUILD_DIR" \
  -DAASDK_LIBRARIES="$AASDK_DIR/lib/libaasdk.so" \
  -DAASDK_PROTO_LIBRARIES="$AASDK_DIR/lib/libaasdk_proto.so"
cmake --build "$OPENAUTO_BUILD_DIR" -j"$(nproc)"

echo "Build complete."
echo "Executing openauto..."
"$OPENAUTO_DIR/bin/autoapp"
