#!/usr/bin/env bash
# Build p47 entirely inside Docker — no local compilers needed.
#
# Produces: p47.exe + bulletml.dll (Windows x64)
#
# Prerequisites: Docker
set -e

rm -f bulletml.dll p47.exe

IMAGE="p47-builder"

echo "=== Building in Docker ==="
docker build -t "$IMAGE" .

echo "=== Extracting artifacts ==="
CONTAINER=$(docker create "$IMAGE")
docker cp "$CONTAINER:/build/out/p47.exe"     ./p47.exe
docker cp "$CONTAINER:/build/out/bulletml.dll" ./bulletml.dll
docker rm "$CONTAINER" >/dev/null

echo "=== Done: p47.exe + bulletml.dll ==="
