use crate::core::vector::Vector2;
use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

extern "C" {
    fn SDL_InitSubSystem(flags: u32) -> c_int;
    fn SDL_NumJoysticks() -> c_int;
    fn SDL_IsGameController(joystick_index: c_int) -> c_int;
    fn SDL_GameControllerOpen(joystick_index: c_int) -> *mut c_void;
    fn SDL_GameControllerGetButton(gamecontroller: *mut c_void, button: c_int) -> u8;
    fn SDL_GameControllerGetAxis(gamecontroller: *mut c_void, axis: c_int) -> i16;
}

// SDL_INIT_GAMECONTROLLER implies SDL_INIT_JOYSTICK.
const SDL_INIT_GAMECONTROLLER: u32 = 0x0000_2000;

// Analog-stick deflection past this magnitude counts as a held direction.
const STICK_DEADZONE: i16 = 16384;

// SDL_GameControllerButton values. The GameController API maps every recognized
// pad onto this abstract, layout-independent set, so we can reference buttons by
// physical position (south/east/west/north, bumpers, d-pad) instead of the
// device-specific numeric indices the raw Joystick API exposes.
const BUTTON_SOUTH: c_int = 0; // A (Xbox) / Cross (PlayStation)
const BUTTON_EAST: c_int = 1; // B (Xbox) / Circle (PlayStation)
const BUTTON_WEST: c_int = 2; // X (Xbox) / Square (PlayStation)
const BUTTON_NORTH: c_int = 3; // Y (Xbox) / Triangle (PlayStation)
const BUTTON_BACK: c_int = 4; // Back / View / Select
const BUTTON_START: c_int = 6; // Start / Menu / Options
const BUTTON_LEFT_BUMPER: c_int = 9;
const BUTTON_RIGHT_BUMPER: c_int = 10;
const BUTTON_DPAD_UP: c_int = 11;
const BUTTON_DPAD_DOWN: c_int = 12;
const BUTTON_DPAD_LEFT: c_int = 13;
const BUTTON_DPAD_RIGHT: c_int = 14;

// SDL_GameControllerAxis values (left analog stick + the analog triggers).
const AXIS_LEFT_X: c_int = 0;
const AXIS_LEFT_Y: c_int = 1;
const AXIS_TRIGGER_LEFT: c_int = 4;
const AXIS_TRIGGER_RIGHT: c_int = 5;

// Triggers report 0..32767; treat a past-halfway pull as a digital press.
const TRIGGER_THRESHOLD: i16 = 16384;

// Full-scale magnitude of a signed SDL axis; divide a raw reading by it to map
// onto [-1, 1].
const AXIS_MAX: f32 = 32767.0;

// Radial deadzone for the analog stick when read as a continuous vector. Smaller
// than STICK_DEADZONE (the digital threshold) so gentle tilts still register.
const STICK_ANALOG_DEADZONE: f32 = 0.2;

// Which controller inputs drive each in-game action. Naming the bindings here
// keeps the mapping obvious and easy to retune. The right trigger doubles for
// fire and the left trigger for special, so either hand can drive either action.
const FIRE_BUTTONS: [c_int; 3] = [BUTTON_SOUTH, BUTTON_WEST, BUTTON_RIGHT_BUMPER];
const SPECIAL_BUTTONS: [c_int; 3] = [BUTTON_EAST, BUTTON_NORTH, BUTTON_LEFT_BUMPER];

// SDL2 SDLK keycode values
const SK_RIGHT: u32 = 1073741903;
const SK_LEFT: u32 = 1073741904;
const SK_DOWN: u32 = 1073741905;
const SK_UP: u32 = 1073741906;
const SK_KP_6: u32 = 1073741918;
const SK_KP_4: u32 = 1073741916;
const SK_KP_2: u32 = 1073741914;
const SK_KP_8: u32 = 1073741920;
const SK_Z: u32 = 122;
const SK_X: u32 = 120;
const SK_LCTRL: u32 = 1073742048;
const SK_LALT: u32 = 1073742050;
const SK_LSHIFT: u32 = 1073742049;
const SK_P: u32 = 112;
const SK_ESCAPE: u32 = 27;

// Indices into KEY_STATE for each tracked key
const IDX_RIGHT: usize = 0;
const IDX_LEFT: usize = 1;
const IDX_DOWN: usize = 2;
const IDX_UP: usize = 3;
const IDX_KP_6: usize = 4;
const IDX_KP_4: usize = 5;
const IDX_KP_2: usize = 6;
const IDX_KP_8: usize = 7;
const IDX_Z: usize = 8;
const IDX_X: usize = 9;
const IDX_LCTRL: usize = 10;
const IDX_LALT: usize = 11;
const IDX_LSHIFT: usize = 12;
const IDX_P: usize = 13;
const IDX_ESCAPE: usize = 14;

// Pad state bitmasks
const PAD_UP: c_int = 1;
const PAD_DOWN: c_int = 2;
const PAD_LEFT: c_int = 4;
const PAD_RIGHT: c_int = 8;
const PAD_BUTTON1: c_int = 16;
const PAD_BUTTON2: c_int = 32;

// Key state table, indexed by our compact key indices above
static mut KEY_STATE: [u8; 15] = [0u8; 15];

static CONTROLLER: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

fn sdl2_keycode_to_index(kc: u32) -> Option<usize> {
    match kc {
        k if k == SK_RIGHT => Some(IDX_RIGHT),
        k if k == SK_LEFT => Some(IDX_LEFT),
        k if k == SK_DOWN => Some(IDX_DOWN),
        k if k == SK_UP => Some(IDX_UP),
        k if k == SK_KP_6 => Some(IDX_KP_6),
        k if k == SK_KP_4 => Some(IDX_KP_4),
        k if k == SK_KP_2 => Some(IDX_KP_2),
        k if k == SK_KP_8 => Some(IDX_KP_8),
        k if k == SK_Z => Some(IDX_Z),
        k if k == SK_X => Some(IDX_X),
        k if k == SK_LCTRL => Some(IDX_LCTRL),
        k if k == SK_LALT => Some(IDX_LALT),
        k if k == SK_LSHIFT => Some(IDX_LSHIFT),
        k if k == SK_P => Some(IDX_P),
        k if k == SK_ESCAPE => Some(IDX_ESCAPE),
        _ => None,
    }
}

/// Called by platform.rs when a SDL2 key event arrives.
pub(crate) fn handle_key_event(keycode: u32, pressed: bool) {
    if let Some(idx) = sdl2_keycode_to_index(keycode) {
        unsafe {
            KEY_STATE[idx] = if pressed { 1 } else { 0 };
        }
    }
}

#[inline]
fn key_at(idx: usize) -> bool {
    unsafe { KEY_STATE[idx] != 0 }
}

/// True when the given SDL_GameControllerButton is held on the open controller.
fn controller_button_held(button: c_int) -> bool {
    let gc = CONTROLLER.load(Ordering::Relaxed);
    !gc.is_null() && unsafe { SDL_GameControllerGetButton(gc, button) != 0 }
}

/// True when the given SDL_GameControllerAxis trigger is pulled past the
/// half-pull threshold on the open controller.
fn trigger_pulled(axis: c_int) -> bool {
    let gc = CONTROLLER.load(Ordering::Relaxed);
    !gc.is_null() && unsafe { SDL_GameControllerGetAxis(gc, axis) > TRIGGER_THRESHOLD }
}

/// Initialize the SDL2 game-controller subsystem and open the first connected
/// controller. Returns 0 on success, -1 on failure.
pub fn pad_open_controller() -> c_int {
    unsafe {
        if SDL_InitSubSystem(SDL_INIT_GAMECONTROLLER) < 0 {
            return -1;
        }
        for i in 0..SDL_NumJoysticks() {
            if SDL_IsGameController(i) == 0 {
                continue;
            }
            let gc = SDL_GameControllerOpen(i);
            if !gc.is_null() {
                CONTROLLER.store(gc, Ordering::Relaxed);
                return 0;
            }
        }
        -1
    }
}

/// Returns a bitmask of directional pad state (UP/DOWN/LEFT/RIGHT), combining
/// the keyboard, the left analog stick, and the controller d-pad.
fn pad_get_pad_state() -> c_int {
    let gc = CONTROLLER.load(Ordering::Relaxed);
    let axis = |a: c_int| if !gc.is_null() { unsafe { SDL_GameControllerGetAxis(gc, a) } } else { 0 };
    let held = |b: c_int| !gc.is_null() && unsafe { SDL_GameControllerGetButton(gc, b) != 0 };

    let x = axis(AXIS_LEFT_X);
    let y = axis(AXIS_LEFT_Y);

    let mut pad: c_int = 0;
    if key_at(IDX_RIGHT) || key_at(IDX_KP_6) || x > STICK_DEADZONE || held(BUTTON_DPAD_RIGHT) {
        pad |= PAD_RIGHT;
    }
    if key_at(IDX_LEFT) || key_at(IDX_KP_4) || x < -STICK_DEADZONE || held(BUTTON_DPAD_LEFT) {
        pad |= PAD_LEFT;
    }
    if key_at(IDX_DOWN) || key_at(IDX_KP_2) || y > STICK_DEADZONE || held(BUTTON_DPAD_DOWN) {
        pad |= PAD_DOWN;
    }
    if key_at(IDX_UP) || key_at(IDX_KP_8) || y < -STICK_DEADZONE || held(BUTTON_DPAD_UP) {
        pad |= PAD_UP;
    }
    pad
}

/// Left analog stick as a ship-space vector (+x = right, +y = up; note SDL's Y
/// axis points down, so it is flipped here). Returns the zero vector inside the
/// deadzone; outside it, the magnitude is rescaled to ramp from 0 at the
/// deadzone edge up to 1 at full tilt, so there is no jump as the stick engages.
fn stick_vector() -> Vector2 {
    let gc = CONTROLLER.load(Ordering::Relaxed);
    if gc.is_null() {
        return Vector2 { x: 0.0, y: 0.0 };
    }
    let x = unsafe { SDL_GameControllerGetAxis(gc, AXIS_LEFT_X) } as f32 / AXIS_MAX;
    let y = -(unsafe { SDL_GameControllerGetAxis(gc, AXIS_LEFT_Y) } as f32 / AXIS_MAX);
    let mag = (x * x + y * y).sqrt();
    if mag <= STICK_ANALOG_DEADZONE {
        return Vector2 { x: 0.0, y: 0.0 };
    }
    let scaled = ((mag - STICK_ANALOG_DEADZONE) / (1.0 - STICK_ANALOG_DEADZONE)).min(1.0);
    Vector2 {
        x: x / mag * scaled,
        y: y / mag * scaled,
    }
}

/// Digital directional sources (keyboard arrows/keypad + controller d-pad) as a
/// ship-space vector, each axis collapsed to -1, 0, or +1.
fn digital_vector() -> Vector2 {
    let mut x = 0.0;
    let mut y = 0.0;
    if key_at(IDX_RIGHT) || key_at(IDX_KP_6) || controller_button_held(BUTTON_DPAD_RIGHT) {
        x += 1.0;
    }
    if key_at(IDX_LEFT) || key_at(IDX_KP_4) || controller_button_held(BUTTON_DPAD_LEFT) {
        x -= 1.0;
    }
    if key_at(IDX_UP) || key_at(IDX_KP_8) || controller_button_held(BUTTON_DPAD_UP) {
        y += 1.0;
    }
    if key_at(IDX_DOWN) || key_at(IDX_KP_2) || controller_button_held(BUTTON_DPAD_DOWN) {
        y -= 1.0;
    }
    Vector2 { x, y }
}

/// Returns a bitmask of button state (BUTTON1/BUTTON2).
fn pad_get_button_state() -> c_int {
    let gc = CONTROLLER.load(Ordering::Relaxed);
    let any_held = |buttons: &[c_int]| -> bool {
        !gc.is_null()
            && buttons
                .iter()
                .any(|&b| unsafe { SDL_GameControllerGetButton(gc, b) != 0 })
    };

    let press1 = key_at(IDX_Z)
        || key_at(IDX_LCTRL)
        || any_held(&FIRE_BUTTONS)
        || trigger_pulled(AXIS_TRIGGER_LEFT);
    let press2 = key_at(IDX_X)
        || key_at(IDX_LALT)
        || key_at(IDX_LSHIFT)
        || any_held(&SPECIAL_BUTTONS)
        || trigger_pulled(AXIS_TRIGGER_RIGHT);

    let mut btn: c_int = 0;
    if press1 {
        btn |= PAD_BUTTON1;
    }
    if press2 {
        btn |= PAD_BUTTON2;
    }
    btn
}

/// Check whether a given SDL2 SDLK keycode is currently pressed.
/// Callers pass 112 (p) and 27 (ESC) — both valid SDL2 ASCII keycodes.
fn pad_is_key_pressed(sk: c_int) -> c_int {
    if sk < 0 {
        return 0;
    }
    match sdl2_keycode_to_index(sk as u32) {
        Some(idx) => {
            if key_at(idx) {
                1
            } else {
                0
            }
        }
        None => 0,
    }
}

// ---------------------------------------------------------------------------
// High-level input queries. These are the public surface of this module; all
// other input logic (key tables, joystick polling, bitmasks) stays private.
// ---------------------------------------------------------------------------

/// True while the fire button (button 1) is held.
pub fn is_fire_button_pressed() -> bool {
    pad_get_button_state() & PAD_BUTTON1 != 0
}

/// True while the special button (button 2) is held.
pub fn is_special_button_pressed() -> bool {
    pad_get_button_state() & PAD_BUTTON2 != 0
}

/// True while the directional up input is active.
pub fn is_up_pressed() -> bool {
    pad_get_pad_state() & PAD_UP != 0
}

/// True while the directional down input is active.
pub fn is_down_pressed() -> bool {
    pad_get_pad_state() & PAD_DOWN != 0
}

/// True while the directional left input is active.
pub fn is_left_pressed() -> bool {
    pad_get_pad_state() & PAD_LEFT != 0
}

/// True while the directional right input is active.
pub fn is_right_pressed() -> bool {
    pad_get_pad_state() & PAD_RIGHT != 0
}

/// True while any directional input is active.
pub fn is_any_direction_pressed() -> bool {
    pad_get_pad_state() != 0
}

/// The current movement direction as a ship-space vector (+x = right, +y = up),
/// fusing the analog stick with the keyboard arrows/keypad and the d-pad.
///
/// Returns `None` when there is no input. Otherwise the vector's length is in
/// `(0, 1]`: the analog stick yields a proportional magnitude (gentle tilt =
/// slower), while digital sources yield a unit vector (diagonals normalized so
/// they are not faster than cardinals). Callers scale it by their own speed.
pub fn pad_get_direction() -> Option<Vector2> {
    let stick = stick_vector();
    let digital = digital_vector();

    // Per axis, let whichever source pushes hardest win, so the stick and the
    // keys/d-pad can be used interchangeably.
    let mut v = Vector2 {
        x: if stick.x.abs() >= digital.x.abs() { stick.x } else { digital.x },
        y: if stick.y.abs() >= digital.y.abs() { stick.y } else { digital.y },
    };

    let mag = (v.x * v.x + v.y * v.y).sqrt();
    if mag <= f32::EPSILON {
        return None;
    }
    // Clamp to unit length: a digital diagonal is sqrt(2) long and must be
    // brought back to 1 (the classic 0.707 per-axis); the stick is already
    // within the unit circle.
    if mag > 1.0 {
        v.x /= mag;
        v.y /= mag;
    }
    Some(v)
}

/// True while pause is requested (keyboard P or the controller Start button).
pub fn is_pause_pressed() -> bool {
    pad_is_key_pressed(SK_P as c_int) != 0 || controller_button_held(BUTTON_START)
}

/// True while quit is requested (keyboard Escape or the controller Back button).
pub fn is_quit_pressed() -> bool {
    pad_is_key_pressed(SK_ESCAPE as c_int) != 0 || controller_button_held(BUTTON_BACK)
}
