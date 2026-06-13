//! Boot / command-line entry (port of `boot()`, `parseArgs()`, `usage()` and
//! `Logger` from the D `P47Boot` module).
//!
//! This is the game's entry point. `main` (in `main.rs`) hands us the program
//! arguments and we own everything from here: open the controller, seed the RNG,
//! parse the command line (calling the relevant setters directly — they all
//! live in this crate) and finally run the main loop.

use crate::core::rand::rand_set_seed;
use crate::game_manager::{
    game_manager_set_no_bonus, game_manager_set_no_field, game_manager_set_nowait,
};
use crate::main_loop::main_loop_run;
use crate::pad::pad_open_controller;
use crate::renderer::renderer_set_brightness;
use crate::ship::ship_set_slow;
use crate::sound::sound_manager_set_no_sound;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

/// Options accumulated while parsing the command line and consumed by
/// `main_loop_run`. The other flags (`-nosound`, `-slowship`, …) take effect
/// immediately through their setters and are not stored here.
struct BootOptions {
    lowres: bool,
    window_mode: bool,
    fullscreen_desktop: bool,
    luminous: f32,
    accframe: i32,
}

impl Default for BootOptions {
    fn default() -> Self {
        BootOptions {
            lowres: false,
            window_mode: false,
            fullscreen_desktop: false,
            luminous: 0.0,
            accframe: 0,
        }
    }
}

/// Port of the D `Logger` class: write info/error messages to stderr.
struct Logger;

impl Logger {
    fn error(msg: &str) {
        eprintln!("Error: {}", msg);
    }
}

/// Mimics C's `atoi`: parse an optional sign followed by leading decimal
/// digits, ignoring any trailing characters, and yield 0 for anything that
/// does not start with a number. The D code relied on this lenient behaviour.
fn c_atoi(s: &str) -> i32 {
    let bytes = s.trim_start().as_bytes();
    let mut i = 0;
    let mut neg = false;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        neg = bytes[i] == b'-';
        i += 1;
    }
    let mut value: i32 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value
            .wrapping_mul(10)
            .wrapping_add((bytes[i] - b'0') as i32);
        i += 1;
    }
    if neg {
        -value
    } else {
        value
    }
}

fn usage(prog: &str) {
    Logger::error(&format!(
        "Usage: {} [-brightness [0-100]] [-luminous [0-100]] [-nosound] [-window] \
         [-fullscreen] [-lowres] [-slowship] [-nowait] [-nofield] [-nobonus]",
        prog
    ));
}

/// Parse the command line, applying the immediate-effect options through their
/// setters and collecting the deferred ones into `BootOptions`. Returns `Err`
/// on an invalid option (after printing usage), matching the D version which
/// threw `Invalid options`.
fn parse_args(args: &[String]) -> Result<BootOptions, ()> {
    let prog = args.first().map(String::as_str).unwrap_or("p47");
    let mut opt = BootOptions::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-brightness" => {
                if i >= args.len() - 1 {
                    usage(prog);
                    return Err(());
                }
                i += 1;
                let b = c_atoi(&args[i]) as f32 / 100.0;
                if b < 0.0 || b > 1.0 {
                    usage(prog);
                    return Err(());
                }
                renderer_set_brightness(b);
            }
            "-luminous" => {
                if i >= args.len() - 1 {
                    usage(prog);
                    return Err(());
                }
                i += 1;
                let l = c_atoi(&args[i]) as f32 / 100.0;
                if l < 0.0 || l > 1.0 {
                    usage(prog);
                    return Err(());
                }
                opt.luminous = l;
            }
            "-nosound" => sound_manager_set_no_sound(1),
            "-window" => opt.window_mode = true,
            "-fullscreen" => opt.fullscreen_desktop = true,
            "-lowres" => opt.lowres = true,
            "-slowship" => ship_set_slow(1),
            "-nowait" => game_manager_set_nowait(1),
            "-nofield" => game_manager_set_no_field(1),
            "-nobonus" => game_manager_set_no_bonus(1),
            "-accframe" => opt.accframe = 1,
            _ => {
                usage(prog);
                return Err(());
            }
        }
        i += 1;
    }

    Ok(opt)
}

/// Seed value derived from the wall clock, replacing the D
/// `MonoTime.currTime.ticks`. Only used to vary the RNG between runs.
fn time_seed() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u32)
        .unwrap_or(0)
}

/// Boot the game. `args` are the program arguments (args[0] is the program
/// name, as in C). Returns `EXIT_SUCCESS` (0) or `EXIT_FAILURE` (1).
pub fn run(args: Vec<String>) -> i32 {
    // openJoystick(); the D code swallowed any error here.
    pad_open_controller();

    rand_set_seed(time_seed());

    let opt = match parse_args(&args) {
        Ok(opt) => opt,
        Err(()) => return EXIT_FAILURE,
    };

    let r = main_loop_run(
        if opt.lowres { 1 } else { 0 },
        if opt.window_mode { 1 } else { 0 },
        if opt.fullscreen_desktop { 1 } else { 0 },
        opt.luminous,
        opt.accframe,
    );

    if r == 0 {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    }
}
