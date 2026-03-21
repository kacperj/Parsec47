FROM mstorsjo/llvm-mingw:latest

ARG LDC_VER=1.42.0

# Install LDC (Linux host) + Windows cross-compilation libs
RUN apt-get update && apt-get install -y --no-install-recommends wget xz-utils p7zip-full && \
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

WORKDIR /build

# ── Rust workspace (mt + renderer → Windows static libs) ──
COPY rust/ rust/

RUN . "$HOME/.cargo/env" && \
    mkdir -p out && \
    cd rust && \
    cargo build --release --target x86_64-pc-windows-gnu && \
    cp target/x86_64-pc-windows-gnu/release/libmt.a /build/out/mt.lib && \
    cp target/x86_64-pc-windows-gnu/release/librenderer.a /build/out/renderer.lib

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
COPY import/    import/
COPY lib/       lib/
COPY resource/  resource/
# Workaround: on Windows both `import SDL_keysym` and `import SDL_Keysym`
# resolve to SDL_keysym.d (case-insensitive FS). On Linux we need a wrapper.
RUN printf 'module SDL_Keysym;\npublic import SDL_keysym;\n' > import/SDL_Keysym.d

RUN set -e && \
    ALL_SRC=$(find import src -name '*.d' | sort | tr '\n' ' ') && \
    ldc2 --mtriple=x86_64-windows-msvc \
      -c -I=import -O --release -d-version=Win32_release -wi \
      -of=out/p47.obj $ALL_SRC && \
    ldc2 --mtriple=x86_64-windows-msvc \
      -of=out/p47.exe out/p47.obj \
      -L=resource/p47.RES \
      -L=lib/SDL.lib -L=lib/SDL_mixer.lib \
      -L=opengl32.lib -L=out/bulletml.lib -L=out/mt.lib -L=out/renderer.lib \
      -L=/SUBSYSTEM:WINDOWS \
      -L=/DEFAULTLIB:user32 && \
    rm -f out/p47.obj
