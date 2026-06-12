//! SDL main loop (port of the D `MainLoop` class).
//!
//! Drives the fixed-timestep frame loop: poll events, derive the frame count
//! from `game_manager_get_interval()`, advance the game manager that many
//! times, then clear / draw / flip. On exit it closes the game manager, saves
//! preferences and tears down SDL. All the work is delegated to functions that
//! already live in this crate, so the loop is pure orchestration.

use crate::game_manager::{
    game_manager_close, game_manager_draw, game_manager_get_interval, game_manager_init,
    game_manager_move, game_manager_start,
};
use crate::platform::{
    window_delay, window_get_resize_h, window_get_resize_w, window_get_ticks, window_poll_events,
};
use crate::prefs::{prefs_load, prefs_save};
use crate::screen::{
    screen_clear, screen_close_sdl, screen_flip, screen_init_sdl, screen_resized,
};

// Maximum number of logic frames simulated per rendered frame (D default; never
// changed via command-line options).
const MAX_SKIP_FRAME: i32 = 5;

/// Run the whole game until quit. Returns 0 on success, nonzero on error
/// (1 = window creation failed, 2 = OpenGL error). The D version threw
/// exceptions in these two cases; here we print to stderr and return a code.
pub fn main_loop_run(
    lowres: i32,
    window_mode: i32,
    fullscreen_desktop: i32,
    luminous: f32,
    accframe: i32,
) -> i32 {
    if screen_init_sdl(lowres, window_mode, fullscreen_desktop, luminous) < 0 {
        eprintln!("Error: Unable to create window");
        return 1;
    }

    // initFirst()
    prefs_load();
    game_manager_init();
    game_manager_start();

    let mut done = false;
    let mut prv_tick_count: i64 = 0;

    while !done {
        let ev_mask = window_poll_events();
        if ev_mask & 1 != 0 {
            done = true;
        }
        if ev_mask & 2 != 0 {
            screen_resized(window_get_resize_w(), window_get_resize_h());
        }

        let now_tick = window_get_ticks() as i64;
        // The interval is constant within an iteration (the game manager only
        // changes it inside game_manager_move, which runs below).
        let interval = game_manager_get_interval() as i64;
        let mut frame = ((now_tick - prv_tick_count) / interval) as i32;
        if frame <= 0 {
            frame = 1;
            window_delay((prv_tick_count + interval - now_tick) as u32);
            if accframe != 0 {
                prv_tick_count = window_get_ticks() as i64;
            } else {
                prv_tick_count += interval;
            }
        } else if frame > MAX_SKIP_FRAME {
            frame = MAX_SKIP_FRAME;
            prv_tick_count = now_tick;
        } else {
            prv_tick_count += frame as i64 * interval;
        }

        for _ in 0..frame {
            if game_manager_move() != 0 {
                done = true;
                break;
            }
        }

        screen_clear();
        game_manager_draw();
        if screen_flip() < 0 {
            // Matches the D throw: skips quitLast(). screen_flip already tore
            // down SDL on GL error.
            eprintln!("Error: OpenGL error");
            return 2;
        }
    }

    // quitLast()
    game_manager_close();
    prefs_save();
    screen_close_sdl();
    0
}
