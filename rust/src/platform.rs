use std::os::raw::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicI32, AtomicPtr, Ordering};
use std::ptr::null_mut;

extern "C" {
    fn SDL_Init(flags: u32) -> c_int;
    fn SDL_CreateWindow(
        title: *const c_char,
        x: c_int,
        y: c_int,
        w: c_int,
        h: c_int,
        flags: u32,
    ) -> *mut c_void;
    fn SDL_DestroyWindow(window: *mut c_void);
    fn SDL_GL_CreateContext(window: *mut c_void) -> *mut c_void;
    fn SDL_GL_DeleteContext(context: *mut c_void);
    fn SDL_GL_SwapWindow(window: *mut c_void);
    fn SDL_ShowCursor(toggle: c_int) -> c_int;
    fn SDL_PollEvent(event: *mut u8) -> c_int;
    fn SDL_GetTicks() -> u32;
    fn SDL_Delay(ms: u32);
}

const SDL_INIT_VIDEO: u32 = 0x0000_0020;
const SDL_INIT_EVENTS: u32 = 0x0000_4000;

const SDL_WINDOW_FULLSCREEN: u32 = 0x0000_0001;
const SDL_WINDOW_OPENGL: u32 = 0x0000_0002;
const SDL_WINDOW_RESIZABLE: u32 = 0x0000_0020;
const SDL_WINDOW_FULLSCREEN_DESKTOP: u32 = 0x0000_1001;

// SDL_WINDOWPOS_CENTERED_MASK | 0
const SDL_WINDOWPOS_CENTERED: c_int = 0x2FFF_0000u32 as c_int;

// SDL2 event type constants
const SDL_QUIT_EVENT: u32 = 0x100;
const SDL_WINDOWEVENT: u32 = 0x200;
const SDL_KEYDOWN: u32 = 0x300;
const SDL_KEYUP: u32 = 0x301;

// SDL_WindowEventID for resize
const SDL_WINDOWEVENT_RESIZED: u8 = 5;

// SDL2 event buffer size (SDL_Event is up to 56 bytes; use 64 for safety)
const EVENT_BUF_SIZE: usize = 64;

static WINDOW: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static GL_CTX: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static RESIZE_W: AtomicI32 = AtomicI32::new(0);
static RESIZE_H: AtomicI32 = AtomicI32::new(0);

/// Create an SDL2 window with an OpenGL context.
/// `fullscreen`: 0 = resizable window, 1 = exclusive fullscreen, 2 = fullscreen desktop (native resolution).
/// `title`: null-terminated C string for the window title.
/// Returns 0 on success, -1 on failure.
pub fn window_init(
    width: c_int,
    height: c_int,
    fullscreen: c_int,
    title: *const c_char,
) -> c_int {
    unsafe {
        if SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) < 0 {
            return -1;
        }

        let flags = SDL_WINDOW_OPENGL
            | match fullscreen {
                2 => SDL_WINDOW_FULLSCREEN_DESKTOP,
                1 => SDL_WINDOW_FULLSCREEN,
                _ => SDL_WINDOW_RESIZABLE,
            };

        let win = SDL_CreateWindow(
            title,
            SDL_WINDOWPOS_CENTERED,
            SDL_WINDOWPOS_CENTERED,
            width,
            height,
            flags,
        );
        if win.is_null() {
            return -1;
        }

        let ctx = SDL_GL_CreateContext(win);
        if ctx.is_null() {
            SDL_DestroyWindow(win);
            return -1;
        }

        WINDOW.store(win, Ordering::Relaxed);
        GL_CTX.store(ctx, Ordering::Relaxed);
        RESIZE_W.store(width, Ordering::Relaxed);
        RESIZE_H.store(height, Ordering::Relaxed);
        0
    }
}

/// Destroy the GL context and window.
pub fn window_close() {
    unsafe {
        let ctx = GL_CTX.swap(null_mut(), Ordering::Relaxed);
        if !ctx.is_null() {
            SDL_GL_DeleteContext(ctx);
        }
        let win = WINDOW.swap(null_mut(), Ordering::Relaxed);
        if !win.is_null() {
            SDL_DestroyWindow(win);
        }
    }
}

/// Swap the OpenGL front and back buffers.
pub fn window_gl_swap() {
    let win = WINDOW.load(Ordering::Relaxed);
    if !win.is_null() {
        unsafe { SDL_GL_SwapWindow(win) };
    }
}

/// Show or hide the mouse cursor. `show` non-zero = show, zero = hide.
pub fn window_show_cursor(show: c_int) {
    unsafe { SDL_ShowCursor(show) };
}

/// Poll all pending SDL2 events.
///
/// Returns a bitmask:
///   bit 0 (1) — quit event received
///   bit 1 (2) — window resize; new dimensions via `window_get_resize_w/h`
///
/// Key events are fed directly to `pad::handle_key_event`.
pub fn window_poll_events() -> c_int {
    let mut result: c_int = 0;
    let mut buf = [0u8; EVENT_BUF_SIZE];

    unsafe {
        while SDL_PollEvent(buf.as_mut_ptr()) != 0 {
            // Read event type as little-endian u32 from bytes 0-3
            let ev_type = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);

            match ev_type {
                SDL_QUIT_EVENT => {
                    result |= 1;
                }
                SDL_WINDOWEVENT => {
                    // SDL_WindowEvent layout:
                    //   0-3:  type (u32)
                    //   4-7:  timestamp (u32)
                    //   8-11: windowID (u32)
                    //   12:   event (u8) — SDL_WindowEventID
                    //   13-15: padding
                    //   16-19: data1 (i32)
                    //   20-23: data2 (i32)
                    let sub = buf[12];
                    if sub == SDL_WINDOWEVENT_RESIZED {
                        let w = i32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
                        let h = i32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
                        if w > 150 && h > 100 {
                            RESIZE_W.store(w, Ordering::Relaxed);
                            RESIZE_H.store(h, Ordering::Relaxed);
                            result |= 2;
                        }
                    }
                }
                SDL_KEYDOWN | SDL_KEYUP => {
                    // SDL_KeyboardEvent layout:
                    //   0-3:  type (u32)
                    //   4-7:  timestamp (u32)
                    //   8-11: windowID (u32)
                    //   12:   state (u8)
                    //   13:   repeat (u8)
                    //   14-15: padding
                    //   16-19: keysym.scancode (i32)
                    //   20-23: keysym.sym / SDLK (i32)
                    //   24-25: keysym.mod (u16)
                    let keycode =
                        u32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]);
                    let pressed = ev_type == SDL_KEYDOWN;
                    crate::pad::handle_key_event(keycode, pressed);
                }
                _ => {}
            }

            buf = [0u8; EVENT_BUF_SIZE];
        }
    }

    result
}

/// Width from the most recent resize event (or initial width if no resize yet).
pub fn window_get_resize_w() -> c_int {
    RESIZE_W.load(Ordering::Relaxed)
}

/// Height from the most recent resize event (or initial height if no resize yet).
pub fn window_get_resize_h() -> c_int {
    RESIZE_H.load(Ordering::Relaxed)
}

/// Milliseconds elapsed since SDL was initialized.
pub fn window_get_ticks() -> u32 {
    unsafe { SDL_GetTicks() }
}

/// Sleep for `ms` milliseconds.
pub fn window_delay(ms: u32) {
    unsafe { SDL_Delay(ms) }
}
