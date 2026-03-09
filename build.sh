#!/usr/bin/env bash
# Build script for p47 using DMD 1.076 (D1) + optlink
set -e

DMD1_INSTALL_DIR="$(pwd)/dmd1"

install_dmd1() {
  if [ -f "$DMD1_INSTALL_DIR/windows/bin/dmd.exe" ]; then
    echo "=== DMD 1.076 already installed at $DMD1_INSTALL_DIR ==="
    return 0
  fi

  echo "=== DMD 1.076 not found. Installing to $DMD1_INSTALL_DIR ==="

  LOCAL_ZIP="$(pwd)/dmd.zip"
  if [ ! -f "$LOCAL_ZIP" ]; then
    echo "ERROR: dmd.zip not found in project directory. Place dmd.zip there and re-run." >&2
    exit 1
  fi

  TMP_DIR="$(mktemp -d)"
  trap "rm -rf '$TMP_DIR'" EXIT

  echo "--- Extracting $LOCAL_ZIP ---"
  if ! command -v unzip &>/dev/null; then
    echo "ERROR: unzip not found. Please install unzip and retry." >&2
    exit 1
  fi

  unzip "$LOCAL_ZIP" -d "$TMP_DIR/dmd1_extracted" || {
    echo "ERROR: Failed to extract zip. File may be corrupt or not a valid zip." >&2
    exit 1
  }

  # Find the bin/dmd.exe inside the extracted tree
  DMD_BIN=$(find "$TMP_DIR/dmd1_extracted" -name "dmd.exe" | head -1)
  if [ -z "$DMD_BIN" ]; then
    echo "ERROR: dmd.exe not found in extracted zip. Extracted contents:" >&2
    find "$TMP_DIR/dmd1_extracted" | head -30 >&2
    exit 1
  fi

  # Copy the entire zip root (one level below the extraction dir) so that
  # the windows/ and src/ subtrees are both preserved. DMD resolves phobos
  # as ../../src/phobos relative to the bin/ directory, so the layout must be:
  #   $DMD1_INSTALL_DIR/windows/bin/dmd.exe
  #   $DMD1_INSTALL_DIR/src/phobos/object.d
  ZIP_ROOT=$(find "$TMP_DIR/dmd1_extracted" -maxdepth 1 -mindepth 1 -type d | head -1)
  mkdir -p "$DMD1_INSTALL_DIR"
  cp -r "$ZIP_ROOT/." "$DMD1_INSTALL_DIR/"

  [ -f "$DMD1_INSTALL_DIR/windows/bin/dmd.exe" ] || {
    echo "ERROR: Installation appeared to succeed but dmd.exe not found at $DMD1_INSTALL_DIR/windows/bin/dmd.exe" >&2
    echo "       Installed tree:" >&2
    find "$DMD1_INSTALL_DIR" | head -30 >&2
    exit 1
  }

  echo "=== DMD 1.076 installed at $DMD1_INSTALL_DIR ==="
}

install_dmd1

DMD="$DMD1_INSTALL_DIR/windows/bin/dmd.exe"
PROJ="$(pwd)"
SRC="$PROJ/src"
IMPORT="$PROJ/import"
LIB="$PROJ/lib"
RESOURCE="$PROJ/resource"
DMD1LIB="$DMD1_INSTALL_DIR/windows/lib"
OUT="p47.exe"

DFLAGS="-c -I$IMPORT -O -release -version=Win32_release"

echo "=== Cleaning previous build artifacts ==="
rm -f "$PROJ/p47.obj"

echo "=== Compiling ==="
ALL_SRC=$(find "$IMPORT" "$SRC" -name "*.d" | sort | tr '\n' ' ')
"$DMD" $DFLAGS -ofp47.obj $ALL_SRC

echo "=== Linking ==="

"$DMD" -of$OUT p47.obj \
  "$RESOURCE/p47.RES" "$RESOURCE/p47.def" \
  "$LIB/SDL.lib" "$LIB/SDL_mixer.lib" "$LIB/opengl32.lib" "$LIB/bulletml.lib"

echo "=== Done: $OUT ==="
