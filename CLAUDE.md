# CLAUDE.md

## Project Overview

PARSEC47 (p47) — a retromodern hispeed shoot-'em-up originally by Kenta Cho. This fork modernized the build toolchain and **fully rewrote the game in Rust**.

The game is now written entirely in **Rust**, as a Cargo workspace with two crates: the game crate `p47rust` (`rust/`) and a pure-Rust BulletML library `bulletml` (`bulletml/`). The original D source and the previous C++ BulletML library have both been removed; the Rust crate owns the OS entry point and produces the executable directly.

## Repository Layout

```
Cargo.toml        Workspace root (members: rust, bulletml; release profile)
rust/             Game crate `p47rust` — all game logic, rendering, audio, input
  src/main.rs     OS entry point (was P47Boot.d) → calls boot::run
  src/boot.rs     Command-line parsing + boot sequence
  src/lib.rs      Module root (game_manager, ship, enemy, bullets, screen, sound, …)
  build.rs        Per-target link config (Windows libunwind static + app icon)
bulletml/         Pure-Rust BulletML parser + runner (port of Kenta Cho's libbulletml)
  src/formula.rs  Embedded-formula lexer/parser/evaluator ($rand/$rank/$N, + - * /)
  src/tree.rs     Node arena types (Name/Type enums, NodeId)
  src/parser.rs   XML parse (roxmltree) + label/ref resolution
  src/runner.rs   The frame-stepped interpreter + AppRunner host trait
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

- **windows**: `p47.exe`, `SDL2.dll`, `SDL2_mixer.dll`
- **linux**: `p47` (SDL2 is a system dependency, not shipped)

**Do not use PowerShell or CMD** to run `build.sh` — use Git Bash.

### Build Toolchain Details

- **Windows** (cross-compiled on the `mstorsjo/llvm-mingw` image):
  - **Rust**: stable, target `x86_64-pc-windows-gnu`, built as a `[[bin]]` (GUI subsystem)
  - SDL2 and SDL2_mixer both from the prebuilt MinGW devel packages (no from-source builds). Music ships as FLAC (pre-decoded from the original OGGs with reference libvorbis), so SDL2_mixer's built-in FLAC decoder is used — no Vorbis decode path at runtime
- **Linux** (native on `debian:bookworm`):
  - **Rust**: stable, host target `x86_64-unknown-linux-gnu`, built as a `[[bin]]`
  - SDL2/SDL2_mixer/OpenGL from distro packages (`libsdl2-dev`, `libsdl2-mixer-dev`, `libgl1-mesa-dev`)

The `bulletml` crate is compiled by Cargo as an ordinary workspace dependency (it pulls in `roxmltree`); no C++ toolchain is involved.

## Running

After building, run the executable from `build/`. The Windows build needs the DLLs beside it (`SDL2.dll`, `SDL2_mixer.dll`); the Linux build needs system SDL2 installed. Both need the asset/BulletML pattern directories.

Command-line options: `-window`, `-fullscreen`, `-nosound`, `-lowres`, `-brightness N`, `-luminous N`, `-reverse`, `-slowship`, `-nowait`, `-nofield`, `-nobonus`.

## Code Conventions

- Rust 2021 edition; modules mirror the original D structure (e.g. `ship`, `enemy`, `bullets`, `stage_manager`).
- Many functions are `#[no_mangle] pub extern "C"` — a legacy of the incremental D→Rust port (they used to be the FFI boundary). They are now internal calls; new code need not follow that convention.
- BulletML is the in-tree `bulletml` crate. The game implements its `AppRunner` trait (see `GameRunner` in `bullets/bullet_actor_pool.rs`) to receive runner callbacks. Parsers/runners are boxed and held as opaque `*mut c_void` in the bullet pool's `Copy` structs (cast back at the boundary); `barrage/mod.rs` owns parser lifetimes.
- Commit messages are short (typically just "Work").

## Key Architecture

- `main.rs` / `boot.rs` — entry point: parse args, open joystick, seed RNG, run the main loop
- `main_loop.rs` — SDL event loop and frame timing
- `game_manager.rs` — main game state machine (title → in-game → game-over → pause)
- `stage_manager.rs` — procedural stage generation
- `bullets/`, `bullet_actor.rs`, `barrage/` — bullet-hell pattern execution driven by the `bulletml` crate
- `bulletml/` (separate crate) — pure-Rust BulletML parser + frame-stepped runner
- `ship.rs` — player ship logic (movement, shooting, roll/lock modes)
- `enemy.rs` / `enemy_type.rs` — enemy definitions and behavior
- `screen.rs`, `renderer.rs`, `rendering/`, `letter_render.rs`, `luminous_screen.rs` — OpenGL rendering
- `sound.rs`, `pad.rs`, `platform.rs` — SDL2 audio, input, and windowing
