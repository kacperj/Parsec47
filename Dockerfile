# Multi-stage build for p47. Select the target with `docker build --target`:
#   --target windows  -> p47.exe + bulletml.dll + SDL2.dll + SDL2_mixer.dll (cross-compiled)
#   --target linux    -> p47 (ELF) + libbulletml.so (native; SDL is a system dep)
# `build.sh <target>` drives this. A bare `docker build .` builds the last
# stage (windows), the historical default.

# =============================================================================
# Linux: native x86_64 build using distro SDL2/OpenGL packages
# =============================================================================
FROM debian:bookworm AS linux

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential clang pkg-config curl ca-certificates \
        libsdl2-dev libsdl2-mixer-dev libgl1-mesa-dev && \
    rm -rf /var/lib/apt/lists/*

# Rust (native host target x86_64-unknown-linux-gnu — no extra target needed)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

WORKDIR /build

# ── bulletml (C++ → libbulletml.so) ──
COPY bulletlib/src/         bulletlib/src/
COPY bulletlib/include/     bulletlib/include/

RUN set -e && mkdir -p out && \
    for f in \
      bulletlib/src/bulletmlparser.cpp \
      bulletlib/src/bulletmlparser-tinyxml.cpp \
      bulletlib/src/bulletmlrunner.cpp \
      bulletlib/src/bulletmlrunnerimpl.cpp \
      bulletlib/src/bulletmltree.cpp \
      bulletlib/src/calc.cpp \
      bulletlib/src/formula-variables.cpp \
      bulletlib/src/tinyxml/tinyxml.cpp \
      bulletlib/src/tinyxml/tinyxmlerror.cpp \
      bulletlib/src/tinyxml/tinyxmlparser.cpp \
      bulletlib/include/bulletml_d.cpp; do \
        name=$(basename "${f%.cpp}"); \
        clang++ -c -O2 -DNDEBUG -fPIC \
          -Wno-register -Wno-writable-strings -Wno-string-plus-int \
          -Ibulletlib/include -Ibulletlib/src -o "out/$name.o" "$f"; \
    done && \
    clang++ -shared -o out/libbulletml.so out/*.o && \
    rm -f out/*.o

# ── p47 (Rust → ELF executable) ──
COPY rust/ rust/
COPY assets/images/ assets/images/

RUN . "$HOME/.cargo/env" && \
    cd rust && \
    BULLETML_LIB_DIR=/build/out \
    cargo build --release && \
    cp target/release/p47 /build/out/p47

# =============================================================================
# Windows: cross-compile with llvm-mingw + distro source builds of SDL deps
# =============================================================================
FROM mstorsjo/llvm-mingw:latest AS windows

RUN apt-get update && apt-get install -y --no-install-recommends wget xz-utils p7zip-full cmake make && \
    rm -rf /var/lib/apt/lists/*

# Install Rust with the Windows cross-compilation target
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    . "$HOME/.cargo/env" && \
    rustup target add x86_64-pc-windows-gnu

# ── SDL2 MinGW development package ──
ARG SDL2_VER=2.30.8
RUN mkdir -p /opt/sdl2 && \
    wget -q "https://github.com/libsdl-org/SDL/releases/download/release-${SDL2_VER}/SDL2-devel-${SDL2_VER}-mingw.tar.gz" && \
    tar xf "SDL2-devel-${SDL2_VER}-mingw.tar.gz" && \
    cp -r "SDL2-${SDL2_VER}/x86_64-w64-mingw32"/* /opt/sdl2/ && \
    rm -rf "SDL2-${SDL2_VER}" "SDL2-devel-${SDL2_VER}-mingw.tar.gz" && \
    x86_64-w64-mingw32-ar rcs /opt/sdl2/lib/libgcc.a && \
    x86_64-w64-mingw32-ar rcs /opt/sdl2/lib/libgcc_eh.a

# ── Cross-compile libogg + libvorbis + SDL2_mixer for x86_64 Windows ──
# Build SDL2_mixer from source with the reference libvorbis decoder
# (stb_vorbis built into pre-built SDL2_mixer produces different output).
RUN cat <<'EOF' > /tmp/mingw-toolchain.cmake
set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_C_COMPILER x86_64-w64-mingw32-clang)
set(CMAKE_RC_COMPILER x86_64-w64-mingw32-windres)
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
EOF

ARG LIBOGG_VER=1.3.5
ARG LIBVORBIS_VER=1.3.7
ARG SDL2_MIXER_VER=2.8.0
RUN wget -q "https://github.com/xiph/ogg/releases/download/v${LIBOGG_VER}/libogg-${LIBOGG_VER}.tar.gz" && \
    tar xf "libogg-${LIBOGG_VER}.tar.gz" && \
    cmake -S "libogg-${LIBOGG_VER}" -B /tmp/ogg-build \
      -DCMAKE_TOOLCHAIN_FILE=/tmp/mingw-toolchain.cmake \
      -DCMAKE_INSTALL_PREFIX=/opt/vorbis \
      -DBUILD_SHARED_LIBS=OFF -DBUILD_TESTING=OFF \
      -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
      -DCMAKE_BUILD_TYPE=Release && \
    cmake --build /tmp/ogg-build -j$(nproc) && \
    cmake --install /tmp/ogg-build && \
    rm -rf "libogg-${LIBOGG_VER}"* /tmp/ogg-build && \
    wget -q "https://github.com/xiph/vorbis/releases/download/v${LIBVORBIS_VER}/libvorbis-${LIBVORBIS_VER}.tar.gz" && \
    tar xf "libvorbis-${LIBVORBIS_VER}.tar.gz" && \
    cmake -S "libvorbis-${LIBVORBIS_VER}" -B /tmp/vorbis-build \
      -DCMAKE_TOOLCHAIN_FILE=/tmp/mingw-toolchain.cmake \
      -DCMAKE_INSTALL_PREFIX=/opt/vorbis \
      -DCMAKE_PREFIX_PATH=/opt/vorbis \
      -DBUILD_SHARED_LIBS=OFF -DBUILD_TESTING=OFF \
      -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
      -DCMAKE_BUILD_TYPE=Release && \
    cmake --build /tmp/vorbis-build -j$(nproc) && \
    cmake --install /tmp/vorbis-build && \
    rm -rf "libvorbis-${LIBVORBIS_VER}"* /tmp/vorbis-build && \
    wget -q "https://github.com/libsdl-org/SDL_mixer/releases/download/release-${SDL2_MIXER_VER}/SDL2_mixer-${SDL2_MIXER_VER}.tar.gz" && \
    tar xf "SDL2_mixer-${SDL2_MIXER_VER}.tar.gz" && \
    cmake -S "SDL2_mixer-${SDL2_MIXER_VER}" -B /tmp/mixer-build \
      -DCMAKE_TOOLCHAIN_FILE=/tmp/mingw-toolchain.cmake \
      -DCMAKE_PREFIX_PATH="/opt/sdl2;/opt/vorbis" \
      -DCMAKE_INSTALL_PREFIX=/opt/sdl2 \
      -DBUILD_SHARED_LIBS=ON \
      -DSDL2MIXER_VORBIS=VORBISFILE \
      -DSDL2MIXER_VORBIS_VORBISFILE_SHARED=OFF \
      -DSDL2MIXER_MP3=OFF \
      -DSDL2MIXER_FLAC=OFF \
      -DSDL2MIXER_MOD=OFF \
      -DSDL2MIXER_MIDI=OFF \
      -DSDL2MIXER_OPUS=OFF \
      -DSDL2MIXER_WAVPACK=OFF \
      -DSDL2MIXER_SAMPLES=OFF \
      -DCMAKE_BUILD_TYPE=Release && \
    cmake --build /tmp/mixer-build -j$(nproc) && \
    cmake --install /tmp/mixer-build && \
    rm -rf "SDL2_mixer-${SDL2_MIXER_VER}"* /tmp/mixer-build /tmp/mingw-toolchain.cmake

WORKDIR /build

# ── bulletml (C++ → Windows DLL) ──
COPY bulletlib/src/         bulletlib/src/
COPY bulletlib/include/     bulletlib/include/
COPY bulletlib/bulletml_api.def bulletlib/

RUN set -e && mkdir -p out && \
    for f in \
      bulletlib/src/bulletmlparser.cpp \
      bulletlib/src/bulletmlparser-tinyxml.cpp \
      bulletlib/src/bulletmlrunner.cpp \
      bulletlib/src/bulletmlrunnerimpl.cpp \
      bulletlib/src/bulletmltree.cpp \
      bulletlib/src/calc.cpp \
      bulletlib/src/formula-variables.cpp \
      bulletlib/src/tinyxml/tinyxml.cpp \
      bulletlib/src/tinyxml/tinyxmlerror.cpp \
      bulletlib/src/tinyxml/tinyxmlparser.cpp \
      bulletlib/include/bulletml_d.cpp; do \
        name=$(basename "${f%.cpp}"); \
        x86_64-w64-mingw32-clang++ -c -O2 -DNDEBUG -DBULLETML_SHARED_LIB \
          -Wno-register -Wno-writable-strings -Wno-string-plus-int \
          -Ibulletlib/include -Ibulletlib/src -o "out/$name.o" "$f"; \
    done && \
    x86_64-w64-mingw32-clang++ -shared -static \
      -o out/bulletml.dll out/*.o bulletlib/bulletml_api.def && \
    rm -f out/*.o

# ── p47 (Rust → Windows EXE) ──
COPY rust/ rust/
COPY assets/images/ assets/images/
COPY resource/ resource/

RUN . "$HOME/.cargo/env" && \
    cd rust && \
    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-clang \
    SDL2_LIB_DIR=/opt/sdl2/lib \
    LIBRARY_PATH=/opt/sdl2/lib \
    cargo build --release --target x86_64-pc-windows-gnu && \
    cp target/x86_64-pc-windows-gnu/release/p47.exe /build/out/ && \
    cp /opt/sdl2/bin/SDL2.dll /build/out/ && \
    cp /opt/sdl2/bin/SDL2_mixer.dll /build/out/
