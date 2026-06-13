# Multi-stage build for p47. Select the target with `docker build --target`:
#   --target windows  -> p47.exe + SDL2.dll + SDL2_mixer.dll (cross-compiled)
#   --target linux    -> p47 (ELF; SDL is a system dep)
# `build.sh <target>` drives this. A bare `docker build .` builds the last
# stage (windows), the historical default.
#
# BulletML is the pure-Rust `bulletml/` crate, built as a normal Cargo workspace
# dependency of `rust/` — no C++ toolchain or shared library involved.

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

# ── p47 (Rust workspace → ELF executable) ──
COPY Cargo.toml ./
COPY bulletml/ bulletml/
COPY rust/ rust/
COPY assets/images/ assets/images/

RUN . "$HOME/.cargo/env" && \
    mkdir -p out && \
    cargo build --release -p p47rust && \
    cp target/release/p47 /build/out/p47

# =============================================================================
# Windows: cross-compile with llvm-mingw + distro source builds of SDL deps
# =============================================================================
FROM mstorsjo/llvm-mingw:latest AS windows

RUN apt-get update && apt-get install -y --no-install-recommends wget && \
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

# ── SDL2_mixer MinGW development package (prebuilt) ──
# Music is shipped as FLAC (lossless), so the prebuilt SDL2_mixer's built-in
# FLAC decoder is used — no libvorbis/stb_vorbis decode path is involved. The
# FLAC files were pre-decoded from the original OGGs with reference libvorbis,
# so playback matches the historical from-source build bit-for-bit.
ARG SDL2_MIXER_VER=2.8.0
RUN wget -q "https://github.com/libsdl-org/SDL_mixer/releases/download/release-${SDL2_MIXER_VER}/SDL2_mixer-devel-${SDL2_MIXER_VER}-mingw.tar.gz" && \
    tar xf "SDL2_mixer-devel-${SDL2_MIXER_VER}-mingw.tar.gz" && \
    cp -r "SDL2_mixer-${SDL2_MIXER_VER}/x86_64-w64-mingw32"/* /opt/sdl2/ && \
    rm -rf "SDL2_mixer-${SDL2_MIXER_VER}" "SDL2_mixer-devel-${SDL2_MIXER_VER}-mingw.tar.gz"

WORKDIR /build

# ── p47 (Rust workspace → Windows EXE) ──
COPY Cargo.toml ./
COPY bulletml/ bulletml/
COPY rust/ rust/
COPY assets/images/ assets/images/
COPY resource/ resource/

RUN . "$HOME/.cargo/env" && \
    mkdir -p out && \
    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-clang \
    SDL2_LIB_DIR=/opt/sdl2/lib \
    LIBRARY_PATH=/opt/sdl2/lib \
    cargo build --release --target x86_64-pc-windows-gnu -p p47rust && \
    cp target/x86_64-pc-windows-gnu/release/p47.exe /build/out/ && \
    cp /opt/sdl2/bin/SDL2.dll /build/out/ && \
    cp /opt/sdl2/bin/SDL2_mixer.dll /build/out/
