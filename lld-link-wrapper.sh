#!/c/msys64/usr/bin/bash
# Wrapper for lld-link that ensures MinGW runtime libs are linked as WHOLEARCHIVE.
# DMD passes them as /DEFAULTLIB: which causes symbol resolution ordering issues.

REAL_LLDLINK="/c/msys64/clang64/bin/lld-link.exe"
MINGW_LIBS_DIR="$(cd "$(dirname "$0")/lib" && pwd -W)"

ARGS=()
for arg in "$@"; do
  case "$arg" in
    /DEFAULTLIB:*libgcc.lib|/DEFAULTLIB:*libgcc_eh.lib)
      lib="${arg#/DEFAULTLIB:}"
      ARGS+=("/WHOLEARCHIVE:$lib")
      ;;
    *)
      ARGS+=("$arg")
      ;;
  esac
done

exec "$REAL_LLDLINK" "${ARGS[@]}"
