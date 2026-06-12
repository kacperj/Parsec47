# CLAUDE.md

## Project Overview

PARSEC47 (p47) — a retromodern hispeed shoot-'em-up originally by Kenta Cho. This fork modernized the build toolchain and **fully rewrote the game in Rust**.

The game is now written entirely in **Rust** (`rust/` crate `p47rust`), with a **C++ BulletML** library (`bulletlib/`) linked as a shared library. The original D source has been removed; the Rust crate owns the OS entry point and produces the executable directly.

## Repository Layout

```
rust/             Rust crate `p47rust` (Cargo) — all game logic, rendering, audio, input
  src/main.rs     OS entry point (was P47Boot.d) → calls boot::run
  src/boot.rs     Command-line parsing + boot sequence
  src/lib.rs      Module root (game_manager, ship, enemy, bullets, screen, sound, …)
  build.rs        Per-target link config (Windows libunwind; Linux bulletml rpath)
bulletlib/        C++ BulletML library (→ bulletml.dll on Windows, libbulletml.so on Linux)
resource/         Windows app icon: p47.ico + p47.rc, compiled by rust/build.rs into p47.exe
assets/           Game assets (images, sounds); BulletML pattern XML directories
```

## Building

The project builds entirely inside Docker — no local compilers needed. Only **Docker** is required.

Run the build script using **Git Bash**:

```bash
./build.sh            # Windows build (default)
./build.sh windows    # explicit Windows build
./build.sh linux      # Linux build
./build.sh --clean    # wipe build/ first (combine with a target if desired)
```

The Dockerfile is multi-stage; `build.sh` selects the stage via `docker build --target <windows|linux>` and extracts the matching artifacts into `build/`:

- **windows**: `p47.exe`, `bulletml.dll`, `SDL2.dll`, `SDL2_mixer.dll`
- **linux**: `p47`, `libbulletml.so` (SDL2 is a system dependency, not shipped)

**Do not use PowerShell or CMD** to run `build.sh` — use Git Bash.

### Build Toolchain Details

- **Windows** (cross-compiled on the `mstorsjo/llvm-mingw` image):
  - **Rust**: stable, target `x86_64-pc-windows-gnu`, built as a `[[bin]]` (GUI subsystem)
  - **C++**: `x86_64-w64-mingw32-clang++` → `bulletml.dll`
  - SDL2 from the MinGW devel package; SDL2_mixer + libogg/libvorbis built from source
- **Linux** (native on `debian:bookworm`):
  - **Rust**: stable, host target `x86_64-unknown-linux-gnu`, built as a `[[bin]]`
  - **C++**: `clang++` → `libbulletml.so`
  - SDL2/SDL2_mixer/OpenGL from distro packages (`libsdl2-dev`, `libsdl2-mixer-dev`, `libgl1-mesa-dev`)

## Running

After building, run the executable from `build/`. The Windows build needs the DLLs beside it (`bulletml.dll`, `SDL2.dll`, `SDL2_mixer.dll`); the Linux build needs `libbulletml.so` beside it (rpath `$ORIGIN`) and system SDL2 installed. Both need the asset/BulletML pattern directories.

Command-line options: `-window`, `-fullscreen`, `-nosound`, `-lowres`, `-brightness N`, `-luminous N`, `-reverse`, `-slowship`, `-nowait`, `-nofield`, `-nobonus`.

## Code Conventions

- Rust 2021 edition; modules mirror the original D structure (e.g. `ship`, `enemy`, `bullets`, `stage_manager`).
- Many functions are `#[no_mangle] pub extern "C"` — a legacy of the incremental D→Rust port (they used to be the FFI boundary). They are now internal calls; new code need not follow that convention.
- The Rust crate links the C++ BulletML library directly (`raw-dylib` on Windows, normal `dylib` on Linux — see `src/barrage/mod.rs`).
- Commit messages are short (typically just "Work").

## Key Architecture

- `main.rs` / `boot.rs` — entry point: parse args, open joystick, seed RNG, run the main loop
- `main_loop.rs` — SDL event loop and frame timing
- `game_manager.rs` — main game state machine (title → in-game → game-over → pause)
- `stage_manager.rs` — procedural stage generation
- `bullets/`, `bullet_actor.rs`, `barrage/` — bullet-hell pattern execution using BulletML
- `ship.rs` — player ship logic (movement, shooting, roll/lock modes)
- `enemy.rs` / `enemy_type.rs` — enemy definitions and behavior
- `screen.rs`, `renderer.rs`, `rendering/`, `letter_render.rs`, `luminous_screen.rs` — OpenGL rendering
- `sound.rs`, `pad.rs`, `platform.rs` — SDL2 audio, input, and windowing
