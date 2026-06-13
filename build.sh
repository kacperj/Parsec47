#!/usr/bin/env bash
# Build p47 entirely inside Docker — no local compilers needed.
#
# Usage: ./build.sh [windows|linux] [--clean]
#
# Produces (in build/):
#   windows (default): p47.exe + SDL2.dll + SDL2_mixer.dll
#   linux:             p47   (SDL2 is a system dependency)
#
# Prerequisites: Docker
set -e

TARGET="windows"
CLEAN=0
for arg in "$@"; do
  case "$arg" in
    windows|linux) TARGET="$arg" ;;
    --clean)       CLEAN=1 ;;
    *) echo "Unknown argument: $arg (expected windows|linux|--clean)" >&2; exit 1 ;;
  esac
done

if [[ "$CLEAN" = "1" ]]; then
  rm -rf build
fi
mkdir -p build/assets

IMAGE="p47-builder-$TARGET"

echo "=== Building in Docker (target: $TARGET) ==="
docker build --target "$TARGET" -t "$IMAGE" .

echo "=== Extracting artifacts ==="
CONTAINER=$(docker create "$IMAGE")
if [[ "$TARGET" = "windows" ]]; then
  docker cp "$CONTAINER:/build/out/p47.exe"         ./build/p47.exe
  docker cp "$CONTAINER:/build/out/SDL2.dll"         ./build/SDL2.dll
  docker cp "$CONTAINER:/build/out/SDL2_mixer.dll"   ./build/SDL2_mixer.dll
else
  docker cp "$CONTAINER:/build/out/p47"              ./build/p47
fi
docker rm "$CONTAINER" >/dev/null

cp -r assets/* build/assets


echo "=== Done ==="
