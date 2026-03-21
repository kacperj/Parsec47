# CLAUDE.md

## Project Overview

PARSEC47 (p47) — a retromodern hispeed shoot-'em-up originally by Kenta Cho. This fork modernizes the build toolchain and incrementally rewrites parts in Rust.

The game is written primarily in **D** (42 source files under `src/`), with a **C++ BulletML** library (`bulletlib/`) and a growing **Rust** layer (`rust/` workspace with `mt` and `renderer` crates).

## Repository Layout

```
src/              D source code (game logic, SDL integration)
  abagames/p47/   Game-specific modules (Ship, Enemy, BulletActor, StageManager…)
  abagames/util/  Engine utilities (Actor pool, Vector, Rand, SDL wrappers, BulletML bindings)
import/           D import files (SDL headers, OpenGL bindings)
lib/              Pre-built Windows .lib files (SDL, SDL_mixer)
rust/             Rust workspace (Cargo)
  mt/             Mersenne Twister RNG replacement
  renderer/       Letter/text rendering replacement
bulletlib/        C++ BulletML library (compiled to bulletml.dll)
resource/         Windows resources (.RES)
sounds/           Sound assets
images/           Image assets
large/ middle*/ small*/ morph*/  BulletML pattern XML files (barrage definitions)
```

## Building

The project builds entirely inside Docker — no local compilers needed. Only **Docker** is required.

Run the build script using **Git Bash**:

```bash
./build.sh
```

This will:
1. Build a Docker image with LDC (D compiler), Rust toolchain, and LLVM/MinGW (C++ cross-compiler)
2. Compile the Rust crates (`mt`, `renderer`) as Windows static libraries
3. Compile `bulletlib` C++ sources into `bulletml.dll`
4. Compile all D sources into `p47.exe`
5. Extract `p47.exe` and `bulletml.dll` into the project root

**Do not use PowerShell or CMD** to run `build.sh` — use Git Bash.

### Build Toolchain Details

- **D compiler**: LDC 1.42.0, cross-compiling to `x86_64-windows-msvc`
- **C++ compiler**: `x86_64-w64-mingw32-clang++` (from `mstorsjo/llvm-mingw` Docker image)
- **Rust**: stable toolchain, target `x86_64-pc-windows-gnu`
- **Output**: Windows x64 binaries

## Running

After building, run `p47.exe` on Windows. Requires the DLLs in the project root (`SDL.dll`, `SDL_mixer.dll`, `bulletml.dll`, etc.) and the BulletML pattern directories.

Command-line options: `-window`, `-nosound`, `-lowres`, `-brightness N`, `-luminous N`, `-reverse`, `-slowship`, `-nowait`.

## Code Conventions

- D source uses module paths matching directory structure (e.g., `abagames.p47.Ship`)
- Class names are PascalCase, methods are camelCase
- The codebase is being incrementally ported: some D modules call into Rust static libs via C ABI (`extern(C)`)
- Commit messages are short (typically just "Work")

## Key Architecture

- `P47GameManager` — main game loop, state machine (title → in-game → game-over)
- `P47Boot` — entry point, initializes SDL, screen, sound, and starts the main loop
- `StageManager` — procedural stage generation
- `BulletActorPool` / `BulletActor` — bullet hell pattern execution using BulletML
- `Ship` — player ship logic (movement, shooting, roll/lock modes)
- `Enemy` / `EnemyType` — enemy definitions and behavior
- `MainLoop` (util) — SDL event loop and frame timing
