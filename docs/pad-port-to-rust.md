# Porting Pad.d to Rust

## Goal

Replace the D-language `Pad.d` module (which used SDL1 directly for keyboard and joystick input)
with a Rust implementation, while the rest of the D application continues to use SDL1
for video and rendering.

## Architecture

```
D application (p47.exe)
├── SDL 1.x  (video, rendering, event pump) — unchanged
├── OpenGL   (rendering)                    — unchanged
└── p47rust.dll (Rust, cdylib)
    ├── pad.rs   — keyboard + joystick input
    ├── sound.rs — audio via SDL2_mixer
    └── ...      — renderer, rand, prefs, etc.
```

The D side (`Pad.d`) becomes a thin wrapper that declares `extern (C)` functions and delegates
all input work to the Rust DLL. The original class API (`getDirectionalPadState`, `getButtonState`,
`openJoystick`) is preserved so no other D modules need changes, except for `P47GameManager.d`
which previously accessed the raw `pad.keys` pointer directly.

## Keyboard Approach: SDL1 Pump → Rust State Table

SDL1 already owns the game window and event pump. Every frame, `MainLoop` calls
`SDL_PollEvent` and forwards the raw `SDL_Event*` to `pad_handle_event`. Rust parses the
event type: on `SDL_KEYDOWN` (type byte = 2) or `SDL_KEYUP` (type byte = 3) it extracts the
`SDLKey sym` field and updates an internal `KEY_STATE: [u8; 512]` array.

All keyboard queries (`pad_get_pad_state`, `pad_get_button_state`, `pad_is_key_pressed`) read
from this array. No SDL2 keyboard involvement at all — avoids the security issue of
`RegisterRawInputDevices` (system-wide keylogger) and the fragility of `SDL_CreateWindowFrom`.

### SDL_KeyboardEvent Memory Layout (SDL 1.2)

```
offset 0:   type  (u8)  — SDL_KEYDOWN=2, SDL_KEYUP=3
offset 1:   which (u8)
offset 2:   state (u8)
offset 3:   padding
offset 4:   keysym.scancode (u8)
offset 5-7: padding
offset 8:   keysym.sym (SDLKey = i32)  ← logical key value used as KEY_STATE index
offset 12:  keysym.mod (u16)
offset 14:  keysym.unicode (u16)
```

### Joystick

SDL2 joystick (`SDL_InitSubSystem(SDL_INIT_JOYSTICK)` + `SDL_JoystickOpen`) is entirely
independent of the event pump and window ownership. `SDL_JoystickGetAxis` and
`SDL_JoystickGetButton` poll state directly. No conflicts with SDL1.

## Key Decisions

### SDL1 SDLKey Keysyms

The `KEY_STATE` array is indexed by SDL1 `SDLKey` keysym values (logical keys):

| Input | SDL1 SDLKey |
|-------|-------------|
| Arrow right | SDLK_RIGHT = 275 |
| Arrow left  | SDLK_LEFT = 276 |
| Arrow down  | SDLK_DOWN = 274 |
| Arrow up    | SDLK_UP = 273 |
| Numpad 6    | SDLK_KP6 = 262 |
| Numpad 4    | SDLK_KP4 = 260 |
| Numpad 2    | SDLK_KP2 = 258 |
| Numpad 8    | SDLK_KP8 = 264 |
| Z           | SDLK_z = 122 |
| X           | SDLK_x = 120 |
| LCtrl       | SDLK_LCTRL = 306 |
| LAlt        | SDLK_LALT = 308 |
| LShift      | SDLK_LSHIFT = 304 |
| P           | SDLK_p = 112 |
| Escape      | SDLK_ESCAPE = 27 |

### Removing `pad.keys` Direct Access

`P47GameManager.d` previously accessed the raw SDL1 key state pointer directly:

```d
if (pad.keys[SDLK_p] == SDL_PRESSED) { ... }
if (pad.keys[SDLK_ESCAPE] == SDL_PRESSED) { ... }
```

This is replaced with two new convenience methods on the `Pad` D class:

```d
bool isPausePressed()  { return pad_is_key_pressed(112) != 0; }  // SDLK_p
bool isEscapePressed() { return pad_is_key_pressed(27) != 0; }   // SDLK_ESCAPE
```

The `Uint8* keys` public field is removed entirely from `Pad.d`.

### No Extra Linking Required

The only SDL2 functions used in `pad.rs` are the joystick ones (`SDL_InitSubSystem`,
`SDL_JoystickOpen`, `SDL_JoystickGetAxis`, `SDL_JoystickGetButton`), which are already
present in `SDL2.dll` linked by the `sdl2` crate for the audio module.

## Files Created/Modified

### New Files

| File | Purpose |
|------|---------|
| `rust/src/pad.rs` | Rust implementation: keyboard state table + SDL2 joystick |

### Modified Files

| File | Change |
|------|--------|
| `rust/src/lib.rs` | Added `pub mod pad;` |
| `src/abagames/util/sdl/Pad.d` | Replaced SDL1 implementation with thin `extern(C)` wrapper |
| `src/abagames/p47/P47GameManager.d` | Replaced 3× `pad.keys[SDLK_*]` with `pad.isPausePressed()` / `pad.isEscapePressed()` |

## Initialization Sequence

```
P47Boot.d
  pad = new Pad
  pad.openJoystick()          → pad_open_joystick()
                                  SDL_InitSubSystem(SDL_INIT_JOYSTICK)
                                  SDL_JoystickOpen(0)
  mainLoop = new MainLoop(...)
  mainLoop.loop()
    screen.initSDL()          → SDL1 creates the OpenGL window
    initFirst()
      Sound.init()            → SDL2 audio initialized
      gameManager.init()
    game loop:
      SDL_PollEvent(&event)   → SDL1 pumps Win32 messages, gets SDL_KEYDOWN/SDL_KEYUP
      input.handleEvent(...)  → pad_handle_event(event)
                                  reads event type + SDLKey sym
                                  updates KEY_STATE array
      gameManager.move()
        pad.getDirectionalPadState()     → pad_get_pad_state()
                                  reads KEY_STATE + SDL_JoystickGetAxis()
        pad.getButtonState()  → pad_get_button_state()
                                  reads KEY_STATE + SDL_JoystickGetButton()
```

## Exported Rust Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `pad_open_joystick` | `() -> c_int` | Init SDL2 joystick subsystem, open device 0 |
| `pad_handle_event` | `(event: *const c_void)` | Parse SDL1 event, update KEY_STATE |
| `pad_get_pad_state` | `() -> c_int` | Directional bitmask (UP=1 DOWN=2 LEFT=4 RIGHT=8) |
| `pad_get_button_state` | `() -> c_int` | Button bitmask (BUTTON1=16 BUTTON2=32) |
| `pad_set_button_reversed` | `(v: c_int)` | Set button-swap flag (`-reverse` CLI option) |
| `pad_get_button_reversed` | `() -> c_int` | Read button-swap flag |
| `pad_is_key_pressed` | `(sk: c_int) -> c_int` | Check arbitrary SDL1 SDLKey keysym |
