FROM mstorsjo/llvm-mingw:latest

ARG LDC_VER=1.42.0

# Install LDC (Linux host) + Windows cross-compilation libs
RUN apt-get update && apt-get install -y --no-install-recommends wget xz-utils p7zip-full cmake make && \
    rm -rf /var/lib/apt/lists/* && \
    wget -q "https://github.com/ldc-developers/ldc/releases/download/v${LDC_VER}/ldc2-${LDC_VER}-linux-x86_64.tar.xz" && \
    tar xf ldc2-*-linux-x86_64.tar.xz -C /opt && \
    ln -s /opt/ldc2-${LDC_VER}-linux-x86_64 /opt/ldc2 && \
    rm ldc2-*-linux-x86_64.tar.xz && \
    wget -q "https://github.com/ldc-developers/ldc/releases/download/v${LDC_VER}/ldc2-${LDC_VER}-windows-x64.7z" && \
    7z x ldc2-*-windows-x64.7z -o/tmp > /dev/null && \
    mkdir -p /opt/ldc2/lib-win64 && \
    cp /tmp/ldc2-${LDC_VER}-windows-x64/lib/*.lib /opt/ldc2/lib-win64/ && \
    cp -r /tmp/ldc2-${LDC_VER}-windows-x64/lib/mingw /opt/ldc2/lib-win64/mingw && \
    rm -rf /tmp/ldc2-* ldc2-*-windows-x64.7z

ENV PATH="/opt/ldc2/bin:$PATH"

# LDC cross-compilation config: point lib-dirs at Windows runtime libs
# (switches and import paths are inherited from the default config)
RUN cat <<'CONF' > /opt/ldc2/etc/ldc2.conf/60-target-windows.conf
"x86_64-.*-windows-msvc":
{
    lib-dirs = [
        "/opt/ldc2/lib-win64",
        "/opt/ldc2/lib-win64/mingw",
    ];
};
CONF

## Install Rust with the Windows cross-compilation target
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

# ── Rust workspace (mt + renderer → static libs, sound → DLL) ──
COPY rust/ rust/
COPY assets/images/ assets/images/

RUN . "$HOME/.cargo/env" && \
    mkdir -p out && \
    cd rust && \
    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-clang \
    SDL2_LIB_DIR=/opt/sdl2/lib \
    LIBRARY_PATH=/opt/sdl2/lib \
    cargo build --release --target x86_64-pc-windows-gnu && \
    cp target/x86_64-pc-windows-gnu/release/libmt.a /build/out/mt.lib && \
    cp target/x86_64-pc-windows-gnu/release/librenderer.a /build/out/renderer.lib && \
    cp target/x86_64-pc-windows-gnu/release/sound.dll /build/out/ && \
    llvm-lib /DEF:sound/sound.def /OUT:/build/out/sound.lib /MACHINE:X64 && \
    cp /opt/sdl2/bin/SDL2.dll /build/out/ && \
    cp /opt/sdl2/bin/SDL2_mixer.dll /build/out/

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
    llvm-lib /DEF:bulletlib/bulletml_api.def /OUT:out/bulletml.lib /MACHINE:X64 && \
    rm -f out/*.o

# ── p47 (D → Windows EXE) ──
COPY src/       src/
COPY lib/       lib/
COPY resource/  resource/

RUN set -e && \
    ALL_SRC=$(find import src -name '*.d' | sort | tr '\n' ' ') && \
    ldc2 --mtriple=x86_64-windows-msvc \
      -c -O --release -d-version=Win32_release -wi \
      -of=out/p47.obj $ALL_SRC && \
    ldc2 --mtriple=x86_64-windows-msvc \
      -of=out/p47.exe out/p47.obj \
      -L=resource/p47.RES \
      -L=lib/SDL.lib \
      -L=opengl32.lib -L=out/bulletml.lib -L=out/mt.lib -L=out/renderer.lib \
      -L=out/sound.lib \
      -L=/SUBSYSTEM:WINDOWS \
      -L=/DEFAULTLIB:user32 && \
    rm -f out/p47.obj
