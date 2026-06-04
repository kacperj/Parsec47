//! SDL screen handler and OpenGL setup for PARSEC47.
//!
//! Ported from `abagames.p47.P47Screen`. Owns the window/projection lifecycle
//! and the global OpenGL render state. The underlying window and luminous-glow
//! work lives in `platform` and `luminous_screen`; this module drives them and
//! manages the OpenGL state machine (blending, projection matrices, clearing).

use crate::luminous_screen;
use crate::platform;
use crate::rendering::gl::*;
use core::ffi::{c_char, c_float, c_int};

const NEAR_PLANE: c_float = 0.1;
const FAR_PLANE: c_float = 1000.0;
const CAPTION: &[u8] = b"PARSEC47\0";

struct Screen {
    width: c_int,
    height: c_int,
    has_luminous: bool,
}

static mut STATE: Option<Screen> = None;

/// Set up one-shot OpenGL render state and (optionally) the luminous screen.
fn init(width: c_int, height: c_int, luminous: c_float) {
    unsafe {
        glLineWidth(1.0);
        glEnable(GL_LINE_SMOOTH);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE);
        glEnable(GL_BLEND);
        glDisable(GL_LIGHTING);
        glDisable(GL_CULL_FACE);
        glDisable(GL_DEPTH_TEST);
        glDisable(GL_TEXTURE_2D);
        glDisable(GL_COLOR_MATERIAL);
    }
    let has_luminous = if luminous > 0.0 {
        luminous_screen::luminous_screen_init(luminous, width, height);
        true
    } else {
        false
    };
    unsafe {
        STATE = Some(Screen {
            width,
            height,
            has_luminous,
        });
    }
}

/// Recompute the viewport and perspective frustum for the current size.
fn screen_resized_gl(width: c_int, height: c_int) {
    unsafe {
        glViewport(0, 0, width, height);
        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        let ratio = NEAR_PLANE * height as c_float / width as c_float;
        glFrustum(
            -NEAR_PLANE as f64,
            NEAR_PLANE as f64,
            -ratio as f64,
            ratio as f64,
            0.1,
            FAR_PLANE as f64,
        );
        glMatrixMode(GL_MODELVIEW);
    }
}

fn do_close_sdl() {
    let has_luminous = unsafe { STATE.as_ref().map_or(false, |s| s.has_luminous) };
    if has_luminous {
        luminous_screen::luminous_screen_close();
    }
    platform::window_show_cursor(1);
    platform::window_close();
    unsafe {
        STATE = None;
    }
}

/// Returns true if an OpenGL error occurred (and tears down the screen).
fn handle_error() -> bool {
    let error = unsafe { glGetError() };
    if error == GL_NO_ERROR {
        return false;
    }
    do_close_sdl();
    true
}

/// Create the window/GL context and initialize render state.
///
/// `lowres` halves the 640x480 resolution. `window_mode`/`fullscreen_desktop`
/// select the display mode. `luminous > 0` enables the glow pass.
/// Returns 0 on success, -1 if window creation failed.
#[no_mangle]
pub extern "C" fn screen_init_sdl(
    lowres: c_int,
    window_mode: c_int,
    fullscreen_desktop: c_int,
    luminous: c_float,
) -> c_int {
    let mut width = 640;
    let mut height = 480;
    if lowres != 0 {
        width /= 2;
        height /= 2;
    }
    let fullscreen = if window_mode != 0 {
        0
    } else if fullscreen_desktop != 0 {
        2
    } else {
        1
    };
    if platform::window_init(width, height, fullscreen, CAPTION.as_ptr() as *const c_char) < 0 {
        return -1;
    }
    unsafe {
        glClearColor(0.0, 0.0, 0.0, 0.0);
    }
    // No luminous state yet during the initial resize.
    unsafe {
        STATE = Some(Screen {
            width,
            height,
            has_luminous: false,
        });
    }
    screen_resized(width, height);
    platform::window_show_cursor(0);
    init(width, height, luminous);
    0
}

#[no_mangle]
pub extern "C" fn screen_resized(width: c_int, height: c_int) {
    let has_luminous = unsafe {
        if let Some(ref mut s) = STATE {
            s.width = width;
            s.height = height;
            s.has_luminous
        } else {
            false
        }
    };
    if has_luminous {
        luminous_screen::luminous_screen_resized(width, height);
    }
    screen_resized_gl(width, height);
}

#[no_mangle]
pub extern "C" fn screen_close_sdl() {
    do_close_sdl();
}

#[no_mangle]
pub extern "C" fn screen_flip() -> c_int {
    if handle_error() {
        return -1;
    }
    platform::window_gl_swap();
    0
}

#[no_mangle]
pub extern "C" fn screen_clear() {
    unsafe {
        glClear(GL_COLOR_BUFFER_BIT);
    }
}

#[no_mangle]
pub extern "C" fn screen_start_render_to_texture() {
    if unsafe { STATE.as_ref().map_or(false, |s| s.has_luminous) } {
        luminous_screen::luminous_screen_start_render_to_texture();
    }
}

#[no_mangle]
pub extern "C" fn screen_end_render_to_texture() {
    if unsafe { STATE.as_ref().map_or(false, |s| s.has_luminous) } {
        luminous_screen::luminous_screen_end_render_to_texture();
    }
}

#[no_mangle]
pub extern "C" fn screen_draw_luminous() {
    if unsafe { STATE.as_ref().map_or(false, |s| s.has_luminous) } {
        luminous_screen::luminous_screen_draw();
    }
}

#[no_mangle]
pub extern "C" fn screen_view_ortho_fixed() {
    unsafe {
        glMatrixMode(GL_PROJECTION);
        glPushMatrix();
        glLoadIdentity();
        glOrtho(0.0, 640.0, 480.0, 0.0, -1.0, 1.0);
        glMatrixMode(GL_MODELVIEW);
        glPushMatrix();
        glLoadIdentity();
    }
}

#[no_mangle]
pub extern "C" fn screen_view_perspective() {
    unsafe {
        glMatrixMode(GL_PROJECTION);
        glPopMatrix();
        glMatrixMode(GL_MODELVIEW);
        glPopMatrix();
    }
}
