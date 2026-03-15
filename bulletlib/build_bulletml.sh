#!/c/msys64/usr/bin/bash
# Build bulletML for DMD2/COFF linker on Windows
#
# Outputs deployed directly to p47/:
#   lib/bulletml.lib      — COFF import library for DMD2's COFF linker
#   bulletml.dll          — 64-bit DLL
#   libgcc_s_dw2-1.dll   \
#   libstdc++-6.dll        } MinGW runtime DLLs (ship alongside p47.exe)
#   libwinpthread-1.dll   /
#
# Toolchain:
#   Compile + link DLL : mingw64 g++ (x86_64-w64-mingw32) — only compiler with C++11 support
#   COFF import library : dlltool (adds _ prefix for i386 cdecl automatically)
set -e

export PATH="/c/msys64/mingw64/bin:$PATH"

BDIR="$(cd "$(dirname "$0")" && pwd)"

mkdir -p "$BDIR/temp"
export TMPDIR="$BDIR/temp"
export TMP="$BDIR/temp"
export TEMP="$BDIR/temp"

GPP="/c/msys64/mingw64/bin/g++.exe"
DLLTOOL="/c/msys64/mingw64/bin/dlltool.exe"
INC="$BDIR/include"
SRC="$BDIR/src"
OUTDIR="$BDIR/bin/Release"

CXXFLAGS="-std=c++11 -O2 -DNDEBUG -DBULLETML_SHARED_LIB -I$INC -I$SRC"

SOURCES=(
  "$SRC/bulletmlparser.cpp"
  "$SRC/bulletmlparser-tinyxml.cpp"
  "$SRC/bulletmlrunner.cpp"
  "$SRC/bulletmlrunnerimpl.cpp"
  "$SRC/bulletmltree.cpp"
  "$SRC/calc.cpp"
  "$SRC/formula-variables.cpp"
  "$SRC/tinyxml/tinyxml.cpp"
  "$SRC/tinyxml/tinyxmlerror.cpp"
  "$SRC/tinyxml/tinyxmlparser.cpp"
  "$INC/bulletml_d.cpp"
)

echo "=== Compiling + Linking: bulletml.dll ==="
mkdir -p "$OUTDIR"
"$GPP" $CXXFLAGS -shared -Wl,--export-all-symbols \
  "${SOURCES[@]}" \
  -o "$OUTDIR/bulletml.dll"

echo "=== Generating COFF import library: bulletml.lib ==="
"$DLLTOOL" --dllname bulletml.dll --input-def "$BDIR/bulletml_api.def" \
  --output-lib "$OUTDIR/bulletml.lib"

echo "=== Deploying to p47 ==="
P47="$BDIR/.."
cp "$OUTDIR/bulletml.dll"              "$P47/bulletml.dll"
cp "$OUTDIR/bulletml.lib"              "$P47/lib/bulletml.lib"
# cp "/c/msys64/mingw64/bin/libgcc_s_dw2-1.dll"  "$P47/libgcc_s_dw2-1.dll"
cp "/c/msys64/mingw64/bin/libstdc++-6.dll"      "$P47/libstdc++-6.dll"
cp "/c/msys64/mingw64/bin/libwinpthread-1.dll"  "$P47/libwinpthread-1.dll"

echo "=== Done ==="
