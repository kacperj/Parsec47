use std::os::raw::{c_int, c_void};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

extern "C" {
    fn SDL_InitSubSystem(flags: u32) -> c_int;
    fn SDL_JoystickOpen(device_index: c_int) -> *mut c_void;
    fn SDL_JoystickGetAxis(joystick: *mut c_void, axis: c_int) -> i16;
    fn SDL_JoystickGetButton(joystick: *mut c_void, button: c_int) -> u8;
}

const SDL_INIT_JOYSTICK: u32 = 0x0000_0200;
const JOYSTICK_AXIS: i16 = 16384;

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

static JOYSTICK: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static BUTTON_REVERSED: AtomicBool = AtomicBool::new(false);

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

/// Initialize SDL2 joystick subsystem and open the first joystick.
/// Returns 0 on success, -1 on failure.
pub fn pad_open_joystick() -> c_int {
    unsafe {
        if SDL_InitSubSystem(SDL_INIT_JOYSTICK) < 0 {
            return -1;
        }
        let joy = SDL_JoystickOpen(0);
        if joy.is_null() {
            return -1;
        }
        JOYSTICK.store(joy, Ordering::Relaxed);
        0
    }
}

/// Returns a bitmask of directional pad state (UP/DOWN/LEFT/RIGHT).
pub fn pad_get_pad_state() -> c_int {
    let joy = JOYSTICK.load(Ordering::Relaxed);
    let x: i16 = if !joy.is_null() {
        unsafe { SDL_JoystickGetAxis(joy, 0) }
    } else {
        0
    };
    let y: i16 = if !joy.is_null() {
        unsafe { SDL_JoystickGetAxis(joy, 1) }
    } else {
        0
    };

    let mut pad: c_int = 0;
    if key_at(IDX_RIGHT) || key_at(IDX_KP_6) || x > JOYSTICK_AXIS {
        pad |= PAD_RIGHT;
    }
    if key_at(IDX_LEFT) || key_at(IDX_KP_4) || x < -JOYSTICK_AXIS {
        pad |= PAD_LEFT;
    }
    if key_at(IDX_DOWN) || key_at(IDX_KP_2) || y > JOYSTICK_AXIS {
        pad |= PAD_DOWN;
    }
    if key_at(IDX_UP) || key_at(IDX_KP_8) || y < -JOYSTICK_AXIS {
        pad |= PAD_UP;
    }
    pad
}

/// Returns a bitmask of button state (BUTTON1/BUTTON2), respecting buttonReversed.
pub fn pad_get_button_state() -> c_int {
    let joy = JOYSTICK.load(Ordering::Relaxed);

    let btn1_joy = !joy.is_null()
        && unsafe {
            SDL_JoystickGetButton(joy, 0) != 0
                || SDL_JoystickGetButton(joy, 3) != 0
                || SDL_JoystickGetButton(joy, 4) != 0
                || SDL_JoystickGetButton(joy, 7) != 0
        };
    let btn2_joy = !joy.is_null()
        && unsafe {
            SDL_JoystickGetButton(joy, 1) != 0
                || SDL_JoystickGetButton(joy, 2) != 0
                || SDL_JoystickGetButton(joy, 5) != 0
                || SDL_JoystickGetButton(joy, 6) != 0
        };

    let press1 = key_at(IDX_Z) || key_at(IDX_LCTRL) || btn1_joy;
    let press2 = key_at(IDX_X) || key_at(IDX_LALT) || key_at(IDX_LSHIFT) || btn2_joy;

    let reversed = BUTTON_REVERSED.load(Ordering::Relaxed);
    let mut btn: c_int = 0;
    if press1 {
        btn |= if !reversed { PAD_BUTTON1 } else { PAD_BUTTON2 };
    }
    if press2 {
        btn |= if !reversed { PAD_BUTTON2 } else { PAD_BUTTON1 };
    }
    btn
}

pub fn pad_set_button_reversed(v: c_int) {
    BUTTON_REVERSED.store(v != 0, Ordering::Relaxed);
}

/// Check whether a given SDL2 SDLK keycode is currently pressed.
/// Callers pass 112 (p) and 27 (ESC) — both valid SDL2 ASCII keycodes.
pub fn pad_is_key_pressed(sk: c_int) -> c_int {
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
