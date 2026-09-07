#!/usr/bin/env bash
# Build the pinned headless Vulkan ICD; no machine-global driver installation.
set -euo pipefail

swiftshader_revision=694585a05946e1ed49b6bd577ca6537cbb57f025
repository_root=$(git rev-parse --show-toplevel)
swiftshader_directory="$repository_root/tmp/swiftshader"
mkdir -p "$swiftshader_directory/source"
git -C "$swiftshader_directory/source" init
git -C "$swiftshader_directory/source" fetch --depth 1 \
  https://swiftshader.googlesource.com/SwiftShader "$swiftshader_revision"
git -C "$swiftshader_directory/source" checkout --detach FETCH_HEAD
test "$(git -C "$swiftshader_directory/source" rev-parse HEAD)" = "$swiftshader_revision"

cmake -S "$swiftshader_directory/source" -B "$swiftshader_directory/build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DSWIFTSHADER_BUILD_TESTS=OFF \
  -DSWIFTSHADER_BUILD_BENCHMARKS=OFF \
  -DSWIFTSHADER_BUILD_PVR=OFF \
  -DSWIFTSHADER_BUILD_WSI_XCB=OFF \
  -DSWIFTSHADER_BUILD_WSI_WAYLAND=OFF
cmake --build "$swiftshader_directory/build" --target vk_swiftshader --parallel 2

swiftshader_platform=$(uname -s)
icd_path="$swiftshader_directory/build/$swiftshader_platform/vk_swiftshader_icd.json"
test -f "$icd_path"
case "$swiftshader_platform" in
  Linux)
    cmake -E copy_if_different "$swiftshader_directory/build/libvk_swiftshader.so" "$swiftshader_directory/build/Linux/libvk_swiftshader.so"
    if [[ -n "${GITHUB_ENV:-}" ]]; then
      echo "VK_ICD_FILENAMES=$icd_path" >> "$GITHUB_ENV"
      echo "VK_DRIVER_FILES=$icd_path" >> "$GITHUB_ENV"
    fi
    ;;
  Darwin)
    # ash loads libvulkan.dylib. SwiftShader also exposes the Vulkan API directly.
    test -f "$swiftshader_directory/build/libvk_swiftshader.dylib"
    ln -sfn libvk_swiftshader.dylib "$swiftshader_directory/build/libvulkan.dylib"
    echo "For macOS smoke, set DYLD_LIBRARY_PATH=$swiftshader_directory/build"
    ;;
  *) echo "Unsupported SwiftShader smoke host: $swiftshader_platform" >&2; exit 1 ;;
esac
mkdir -p "$repository_root/tmp/gpu-smoke"
{
  echo "SwiftShader revision: $swiftshader_revision"
  uname -a
  cmake --version
  "${CXX:-c++}" --version
} > "$repository_root/tmp/gpu-smoke/software-environment.log"
echo "Built SwiftShader ICD: $icd_path"
