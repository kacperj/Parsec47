# Porting Sound.d to Rust

## Goal

Replace the D-language `Sound.d` module (which used SDL_mixer 1.x directly) with a Rust implementation using the `sdl2` crate and SDL2_mixer, while keeping the rest of the D application on legacy SDL 1.x for video/input.

## Architecture

```
D application (p47.exe)
├── SDL 1.x  (video, input) — unchanged
├── OpenGL   (rendering)    — unchanged
└── sound.dll (Rust, cdylib)
    ├── sdl2 crate (safe Rust API)
    ├── SDL2.dll
    └── SDL2_mixer.dll (custom-built with libvorbis)
```

The D side (`Sound.d`) becomes a thin wrapper that declares `extern (C)` functions and delegates all audio work to the Rust DLL. The original class API is preserved so no other D modules need changes.

## Key Decision: cdylib (DLL) Instead of staticlib

The existing Rust crates (`mt`, `renderer`) are `no_std` static libraries. The `sound` crate cannot be `no_std` because the `sdl2` Rust crate requires `std`.

Using `std` with the `x86_64-pc-windows-gnu` Rust target introduces MinGW C Runtime (CRT) dependencies (`__mingw_vsnprintf`, etc.) that are **incompatible** with LDC's MSVC linker (`lld-link`). Linking a `std`-using Rust staticlib into an MSVC-linked D executable produces unresolvable symbol mismatches.

**Solution**: Build as a `cdylib` (DLL). Rust handles all internal linking (including `std`, SDL2, SDL2_mixer) within the DLL boundary. The D executable links against an MSVC-compatible import library (`sound.lib`) generated from a `.def` file via `llvm-lib`.

## Files Created/Modified

### New Files

| File | Purpose |
|------|---------|
| `rust/sound/Cargo.toml` | Crate config: `cdylib`, depends on `sdl2` with `mixer` feature |
| `rust/sound/src/lib.rs` | Core implementation: slot-based sound management via FFI |
| `rust/sound/build.rs` | Links `libunwind` statically (see below) |
| `rust/sound/sound.def` | DLL export definitions for `llvm-lib` import library generation |

### Modified Files

| File | Change |
|------|--------|
| `rust/Cargo.toml` | Added `"sound"` to workspace members |
| `src/abagames/util/sdl/Sound.d` | Replaced SDL_mixer imports with `extern (C)` FFI declarations |
| `Dockerfile` | Added SDL2 dev packages, libogg/libvorbis/SDL2_mixer from-source builds, Rust DLL build steps |
| `build.sh` | Extracts `sound.dll`, `SDL2.dll`, `SDL2_mixer.dll` from Docker |
| `.gitignore` | Added `sound.dll`, `SDL2.dll`, `SDL2_mixer.dll` |

## Build Pipeline (Docker)

1. **SDL2 MinGW dev package** — provides headers, import libs, and `SDL2.dll`
2. **libogg 1.3.5** — cross-compiled from source as a static library
3. **libvorbis 1.3.7** — cross-compiled from source as a static library
4. **SDL2_mixer 2.8.0** — cross-compiled from source as a shared DLL with:
   - `SDL2MIXER_VORBIS=VORBISFILE` (reference libvorbis decoder, not stb_vorbis)
   - `SDL2MIXER_VORBIS_VORBISFILE_SHARED=OFF` (statically linked into SDL2_mixer.dll)
   - All other codecs disabled (MP3, FLAC, MOD, MIDI, Opus, WavPack)
5. **Rust workspace** — builds `mt` (staticlib), `renderer` (staticlib), `sound` (cdylib)
6. **Import library** — `llvm-lib /DEF:sound.def /OUT:sound.lib /MACHINE:X64`
7. **D compilation** — LDC links `p47.exe` against `sound.lib` (import lib)

### Cross-compilation Toolchain

- **Rust target**: `x86_64-pc-windows-gnu` with `x86_64-w64-mingw32-clang` linker
- **C/C++ cross-compiler**: `x86_64-w64-mingw32-clang` (from `mstorsjo/llvm-mingw`)
- **D compiler**: LDC 1.42.0, target `x86_64-windows-msvc`

## Linker Issues Encountered and Resolved

### 1. Missing SDL2 libraries (`-lSDL2`, `-lSDL2_mixer`)

**Cause**: Rust linker couldn't find SDL2 and SDL2_mixer during `cdylib` linking.

**Fix**: Set `LIBRARY_PATH=/opt/sdl2/lib` in the Dockerfile's Rust build step.

### 2. Missing GCC runtime (`-lgcc_eh`, `-lgcc`)

**Cause**: Rust's `x86_64-pc-windows-gnu` target implicitly links against `libgcc_eh.a` and `libgcc.a`, but the LLVM-MinGW toolchain doesn't ship these (it uses libunwind instead).

**Fix**: Create empty stub archives:
```bash
x86_64-w64-mingw32-ar rcs /opt/sdl2/lib/libgcc.a
x86_64-w64-mingw32-ar rcs /opt/sdl2/lib/libgcc_eh.a
```

### 3. Undefined `_Unwind_*` symbols

**Cause**: After stubbing `libgcc_eh`, the actual unwind symbols (`_Unwind_Resume`, `_GCC_specific_handler`) were unresolved. LLVM-MinGW provides these in `libunwind`, not `libgcc_eh`.

**Fix**: Added `build.rs` with:
```rust
println!("cargo:rustc-link-lib=static=unwind");
```

The `static=` qualifier is critical — without it, `libunwind.dll` becomes a runtime dependency.

### 4. `libunwind.dll` runtime dependency

**Cause**: Dynamic linking of libunwind meant `sound.dll` imported `libunwind.dll`, which doesn't exist on the target Windows system.

**Fix**: Changed to `static=unwind` in `build.rs` (see above).

### 5. Stale `SDL_mixer.lib` reference

**Cause**: The Dockerfile still had `-L=lib/SDL_mixer.lib` from the original D build, and `import/SDL_mixer.d` was still being compiled (containing inline functions referencing old SDL_mixer 1.x symbols).

**Fix**: Removed `-L=lib/SDL_mixer.lib` from LDC flags. Excluded `SDL_mixer.d` from compilation:
```bash
ALL_SRC=$(find import src -name '*.d' ! -name 'SDL_mixer.d' | sort | tr '\n' ' ')
```

## Audio Quality Issues

### Original Parameters (from git history)

The original `Sound.d` called:
```d
Mix_OpenAudio(44100, AUDIO_S16, 1, 4096)
// 44100 Hz, signed 16-bit little-endian, mono, 4096-sample buffer
```

The OGG music files are: **44100 Hz, mono, 96 kbps, 60-72 seconds**.

### Attempt 1: `mixer::open_audio` with matching parameters

```rust
mixer::open_audio(44100, AUDIO_S16LSB, 1, 4096)
```

**Result**: WAV sound effects perfect. OGG music sounds "odd".

### Attempt 2: Stereo + smaller buffer

```rust
mixer::open_audio(44100, AUDIO_S16LSB, 2, 2048)
```

**Result**: OGG improved but WAV sound effects got "strange fading". Reverted.

### Attempt 3: `Mix_OpenAudioDevice` with `allowed_changes=0`

**Discovery**: SDL2_mixer's `Mix_OpenAudio` internally calls `Mix_OpenAudioDevice` with `SDL_AUDIO_ALLOW_FREQUENCY_CHANGE | SDL_AUDIO_ALLOW_CHANNELS_CHANGE`. This means SDL2 can silently adjust the requested format to match hardware, causing the actual mixing to happen at a different rate/channel count than specified. SDL_mixer 1.x did not have this behavior.

```rust
extern "C" {
    fn Mix_OpenAudioDevice(
        frequency: c_int, format: u16, channels: c_int,
        chunksize: c_int, device: *const c_char, allowed_changes: c_int,
    ) -> c_int;
}

Mix_OpenAudioDevice(44100, AUDIO_S16LSB, 1, 4096, std::ptr::null(), 0)
// allowed_changes=0 forces exact parameters
```

**Result**: WAV still perfect. OGG still sounds "odd".

### Attempt 4: Load OGGs as Chunks instead of Music

**Hypothesis**: SDL2_mixer's `Music` streaming pipeline (used by `Mix_LoadMUS` / `Music::from_file`) processes OGGs differently from the `Chunk` pipeline. Since WAVs (loaded as Chunks) sound correct, forcing OGGs through the same Chunk path should produce identical mixing.

Changed `sound_load_music` to use `Chunk::from_file` and plays on a dedicated `MUSIC_CHANNEL` (channel 8) with infinite looping.

**Result**: Still sounds "odd". This eliminated the Music vs Chunk pipeline as the cause.

### Attempt 5: Provide external libvorbis DLLs

**Hypothesis**: SDL2_mixer's built-in `stb_vorbis` decoder (a minimal single-header decoder) produces different output from the reference `libvorbis` used by SDL_mixer 1.x.

Cross-compiled `libogg.dll`, `libvorbis.dll`, `libvorbisfile.dll` from source and placed alongside `SDL2_mixer.dll`.

**Result**: Still sounds "odd". The pre-built `SDL2_mixer.dll` was compiled with `SDL2MIXER_VORBIS=STB` (stb_vorbis built-in), so it never attempts to dynamically load external vorbis libraries.

### Attempt 6 (current): Build SDL2_mixer from source with libvorbis

**Hypothesis**: Same as attempt 5, but this time SDL2_mixer itself is rebuilt from source with the reference decoder compiled in.

Built SDL2_mixer 2.8.0 from source with:
- `SDL2MIXER_VORBIS=VORBISFILE` — uses libvorbis instead of stb_vorbis
- `SDL2MIXER_VORBIS_VORBISFILE_SHARED=OFF` — statically links libvorbis into SDL2_mixer.dll
- libogg 1.3.5 and libvorbis 1.3.7 cross-compiled as static libraries

**Status**: Worked, but required building libogg + libvorbis + SDL2_mixer from
source in the Windows Docker stage — a slow, heavy step that made the Windows
build far more complex than the Linux one.

### Attempt 7 (current): Pre-decode OGG → FLAC with reference libvorbis

**Hypothesis**: The "odd" sound is purely a *decoder* difference for the same
compressed source. If the OGGs are decoded **once, offline, with reference
libvorbis** (`oggdec` from vorbis-tools) and stored in a **lossless** container,
the stored PCM *is* exactly what the from-source build produced. Any player then
reproduces it bit-for-bit, and no Vorbis decoder is needed at runtime at all.

Converted `ptn0..3.ogg` → `ptn0..3.flac` (libvorbis decode → FLAC encode). The
4 music tracks are now FLAC; everything else was already WAV. `BGM_FILES` and the
`mixer::init` flag (`InitFlag::FLAC`) were updated accordingly.

**Result**: The Windows Docker stage drops the entire libogg/libvorbis/SDL2_mixer
from-source build and uses the **prebuilt `SDL2_mixer-devel` MinGW package** (its
built-in FLAC decoder is statically linked — no companion DLLs). The Windows
block now mirrors the SDL2 block (~8 lines) and the build is much faster. Linux
playback is unchanged (same libvorbis PCM, now via FLAC). This is option 3 from
the "Remaining Theories" list below, refined to FLAC instead of WAV for size.

### Summary of What Works

| Audio type | Format | Status |
|-----------|--------|--------|
| Sound effects (WAV) | 44100 Hz, various | Works perfectly, matches original |
| Background music (OGG) | 44100 Hz, mono, 96 kbps | Sounds "odd" compared to original SDL_mixer 1.x |

### Remaining Theories (if attempt 6 fails)

1. **Decoder precision differences** — Even with reference libvorbis, compiler flags or floating-point behavior could produce slightly different output when cross-compiled with clang vs the original GCC-built libvorbis
2. **SDL2_mixer mixing pipeline** — The internal mixing/resampling in SDL2_mixer 2.x may differ from SDL_mixer 1.x even with identical decoders
3. **Convert OGGs to WAV** — Pre-decode OGG files to WAV during build, eliminating the OGG decoder entirely (at the cost of larger file size, ~5 MB per 60-second track)
4. **Use SDL_mixer 1.x from Rust** — Abandon SDL2 for audio and bind to the same SDL_mixer 1.x library the D code originally used

## Runtime Dependencies

```
p47.exe
├── SDL.dll          (SDL 1.x, original, for video/input)
├── bulletml.dll     (C++ BulletML library)
├── sound.dll        (Rust sound crate)
├── SDL2.dll         (SDL2, for audio subsystem)
└── SDL2_mixer.dll   (SDL2_mixer, custom-built with libvorbis)
```

## Rust Implementation Details

### Global State Management

The crate uses atomic types for thread-safe globals (even though the game is single-threaded, Rust requires `Sync` for statics):

- `AtomicBool` for `NO_SOUND` flag
- `AtomicI32` for `FADE_OUT_SPEED`
- `AtomicPtr<Vec<SoundSlot>>` for the slot array

### SDL2 Context Lifetime

SDL2's context objects (`Sdl`, `AudioSubsystem`, `Sdl2MixerContext`) own resources and clean up on drop. Since the game expects audio to persist for its entire lifetime, `std::mem::forget` is used to prevent premature cleanup:

```rust
std::mem::forget(sdl);
std::mem::forget(audio);
```

### Slot System

Each `Sound` instance in D allocates a slot via `sound_alloc_slot()`. Slots store either a music chunk or a sound effect chunk with its assigned channel number. This mirrors the original D design where each `Sound` object held either a `Mix_Music*` or a `Mix_Chunk*`.
