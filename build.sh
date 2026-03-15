#!/usr/bin/env bash
# Build script for p47 using DMD 2.x (D2)
set -e

DMD2_INSTALL_DIR="$(pwd)/dmd2"
DMD2_VERSION="2.109.1"

install_dmd2() {
  if [ -f "$DMD2_INSTALL_DIR/windows/bin/dmd.exe" ]; then
    echo "=== DMD2 already installed at $DMD2_INSTALL_DIR ==="
    return 0
  fi

  echo "=== Installing DMD2 version $DMD2_VERSION ==="
  ZIP_FILE="$(pwd)/dmd2.zip"

  unzip -q "$ZIP_FILE" -d . || true

  [ -f "$DMD2_INSTALL_DIR/windows/bin/dmd.exe" ] || {
    echo "ERROR: dmd.exe not found after install" >&2
    find "$DMD2_INSTALL_DIR" | head -20 >&2
    exit 1
  }

  echo "=== DMD2 installed at $DMD2_INSTALL_DIR ==="
}

cd bulletlib
./build_bulletml.sh
cd ..
install_dmd2

DMD="$DMD2_INSTALL_DIR/windows/bin/dmd.exe"
PROJ="$(pwd)"
SRC="$PROJ/src"
IMPORT="$PROJ/import"
LIB="$PROJ/lib"
RESOURCE="$PROJ/resource"
OUT="p47.exe"

DFLAGS="-c -I$IMPORT -O -release -version=Win32_release -wi"

echo "=== DMD2 version ==="
"$DMD" --version

echo "=== Compiling ==="
ALL_SRC=$(find "$IMPORT" "$SRC" -name "*.d" | sort | tr '\n' ' ')
"$DMD" $DFLAGS -ofp47.obj $ALL_SRC 2>&1 | head -100; true

echo "=== Compile step done ==="

"$DMD" -of$OUT p47.obj \
  "$RESOURCE/p47.RES" "$RESOURCE/p47.def" \
  "$LIB/SDL.lib" "$LIB/SDL_mixer.lib" "$LIB/opengl32.lib" "$LIB/bulletml.lib" \
  -L/DEFAULTLIB:user32.lib \
  -L/FORCE:MULTIPLE
