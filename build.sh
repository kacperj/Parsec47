#!/usr/bin/env bash
# Build p47 entirely inside Docker — no local compilers needed.
#
# Produces: p47.exe + bulletml.dll + sound.dll + SDL2.dll + SDL2_mixer.dll (Windows x64)
#
# Prerequisites: Docker
set -e

rm -f bulletml.dll sound.dll p47.exe SDL2.dll SDL2_mixer.dll

IMAGE="p47-builder"

echo "=== Building in Docker ==="
docker build -t "$IMAGE" .

echo "=== Extracting artifacts ==="
CONTAINER=$(docker create "$IMAGE")
docker cp "$CONTAINER:/build/out/p47.exe"         ./p47.exe
docker cp "$CONTAINER:/build/out/bulletml.dll"     ./bulletml.dll
docker cp "$CONTAINER:/build/out/sound.dll"        ./sound.dll
docker cp "$CONTAINER:/build/out/SDL2.dll"         ./SDL2.dll
docker cp "$CONTAINER:/build/out/SDL2_mixer.dll"   ./SDL2_mixer.dll
docker rm "$CONTAINER" >/dev/null

echo "=== Done ==="
