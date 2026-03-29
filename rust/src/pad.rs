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

// SDL_KEYDOWN / SDL_KEYUP event type bytes (SDL1)
const SDL_KEYDOWN: u8 = 2;
const SDL_KEYUP: u8 = 3;

// SDL1 SDLKey keysym values (indices into KEY_STATE)
const SK_RIGHT: usize = 275;
const SK_LEFT: usize = 276;
const SK_DOWN: usize = 274;
const SK_UP: usize = 273;
const SK_KP_6: usize = 262;
const SK_KP_4: usize = 260;
const SK_KP_2: usize = 258;
const SK_KP_8: usize = 264;
const SK_Z: usize = 122;
const SK_X: usize = 120;
const SK_LCTRL: usize = 306;
const SK_LALT: usize = 308;
const SK_LSHIFT: usize = 304;

// Pad state bitmasks (must match D-side constants)
const PAD_UP: c_int = 1;
const PAD_DOWN: c_int = 2;
const PAD_LEFT: c_int = 4;
const PAD_RIGHT: c_int = 8;
const PAD_BUTTON1: c_int = 16;
const PAD_BUTTON2: c_int = 32;

// Key state table indexed by SDL1 SDLKey value (SDLK_LAST = 323, so 512 is safe)
static mut KEY_STATE: [u8; 512] = [0u8; 512];

static JOYSTICK: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static BUTTON_REVERSED: AtomicBool = AtomicBool::new(false);

/// Initialize SDL2 joystick subsystem and open the first joystick.
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub extern "C" fn pad_open_joystick() -> c_int {
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

/// Update internal key state from an SDL1 SDL_Event.
///
/// SDL_KeyboardEvent memory layout (SDL 1.2):
///   offset 0: type  (u8) — SDL_KEYDOWN=2 or SDL_KEYUP=3
///   offset 1: which (u8)
///   offset 2: state (u8)
///   offset 3: padding
///   offset 4: keysym.scancode (u8)
///   offset 5-7: padding
///   offset 8: keysym.sym (SDLKey = i32) <- the logical key value
#[no_mangle]
pub extern "C" fn pad_handle_event(event: *const c_void) {
    if event.is_null() {
        return;
    }
    unsafe {
        let bytes = event as *const u8;
        let event_type = *bytes;
        if event_type == SDL_KEYDOWN || event_type == SDL_KEYUP {
            let sym = *(bytes.add(8) as *const i32);
            if sym >= 0 && (sym as usize) < KEY_STATE.len() {
                KEY_STATE[sym as usize] = if event_type == SDL_KEYDOWN { 1 } else { 0 };
            }
        }
    }
}

#[inline]
fn key_pressed(sk: usize) -> bool {
    unsafe { KEY_STATE[sk] != 0 }
}

/// Returns a bitmask of directional pad state (UP/DOWN/LEFT/RIGHT).
#[no_mangle]
pub extern "C" fn pad_get_pad_state() -> c_int {
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
    if key_pressed(SK_RIGHT) || key_pressed(SK_KP_6) || x > JOYSTICK_AXIS {
        pad |= PAD_RIGHT;
    }
    if key_pressed(SK_LEFT) || key_pressed(SK_KP_4) || x < -JOYSTICK_AXIS {
        pad |= PAD_LEFT;
    }
    if key_pressed(SK_DOWN) || key_pressed(SK_KP_2) || y > JOYSTICK_AXIS {
        pad |= PAD_DOWN;
    }
    if key_pressed(SK_UP) || key_pressed(SK_KP_8) || y < -JOYSTICK_AXIS {
        pad |= PAD_UP;
    }
    pad
}

/// Returns a bitmask of button state (BUTTON1/BUTTON2), respecting buttonReversed.
#[no_mangle]
pub extern "C" fn pad_get_button_state() -> c_int {
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

    let press1 = key_pressed(SK_Z) || key_pressed(SK_LCTRL) || btn1_joy;
    let press2 = key_pressed(SK_X) || key_pressed(SK_LALT) || key_pressed(SK_LSHIFT) || btn2_joy;

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

#[no_mangle]
pub extern "C" fn pad_set_button_reversed(v: c_int) {
    BUTTON_REVERSED.store(v != 0, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn pad_get_button_reversed() -> c_int {
    if BUTTON_REVERSED.load(Ordering::Relaxed) {
        1
    } else {
        0
    }
}

/// Check whether a given SDL1 SDLKey keysym is currently pressed.
#[no_mangle]
pub extern "C" fn pad_is_key_pressed(sk: c_int) -> c_int {
    if sk < 0 || (sk as usize) >= 512 {
        return 0;
    }
    if key_pressed(sk as usize) {
        1
    } else {
        0
    }
}
