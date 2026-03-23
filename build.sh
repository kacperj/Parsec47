#!/usr/bin/env bash
# Build p47 entirely inside Docker — no local compilers needed.
#
# Produces: p47.exe + bulletml.dll + sound.dll + prefs.dll + SDL2.dll + SDL2_mixer.dll (Windows x64)
#
# Prerequisites: Docker
set -e

rm -rf build
mkdir -p build/assets

IMAGE="p47-builder"

echo "=== Building in Docker ==="
docker build -t "$IMAGE" .

echo "=== Extracting artifacts ==="
CONTAINER=$(docker create "$IMAGE")
docker cp "$CONTAINER:/build/out/p47.exe"         ./build/p47.exe
docker cp "$CONTAINER:/build/out/bulletml.dll"     ./build/bulletml.dll
docker cp "$CONTAINER:/build/out/p47rust.dll"        ./build/p47rust.dll
docker cp "$CONTAINER:/build/out/SDL2.dll"         ./build/SDL2.dll
docker cp "$CONTAINER:/build/out/SDL2_mixer.dll"   ./build/SDL2_mixer.dll
docker rm "$CONTAINER" >/dev/null

cp -r assets/* build/assets
cp SDL.dll build/SDL.dll


echo "=== Done ==="
