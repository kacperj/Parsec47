//! A pure-Rust port of Kenta Cho's C++ BulletML library (`libbulletml`).
//!
//! BulletML is an XML dialect describing bullet-hell barrage patterns. This crate
//! parses a pattern into a [`BulletMLParser`] (a long-lived, immutable node tree)
//! and steps one bullet through it with a [`BulletMLRunner`]. The runner is driven
//! one game frame at a time and calls back into the host game through the
//! [`AppRunner`] trait to read the bullet's current state (direction, speed, aim)
//! and to enact effects (spawn bullets, change direction/speed, accelerate, vanish).
//!
//! The shape mirrors the original C++ classes: `BulletMLParser`,
//! `BulletMLRunner`/`BulletMLRunnerImpl`, `BulletMLState`, and the `BulletMLRunner`
//! virtual-method interface (here the [`AppRunner`] trait). Angles are in degrees,
//! measured clockwise from straight up, exactly as the C++ engine expects.
//!
//! ```no_run
//! use bulletml::{BulletMLParser, BulletMLRunner, AppRunner, BulletMLState};
//! # struct Game;
//! # impl AppRunner for Game {
//! #     fn get_bullet_direction(&mut self) -> f64 { 0.0 }
//! #     fn get_aim_direction(&mut self) -> f64 { 0.0 }
//! #     fn get_bullet_speed(&mut self) -> f64 { 1.0 }
//! #     fn get_rank(&mut self) -> f64 { 0.5 }
//! #     fn create_simple_bullet(&mut self, _d: f64, _s: f64) {}
//! #     fn create_bullet(&mut self, _st: BulletMLState, _d: f64, _s: f64) {}
//! #     fn get_turn(&mut self) -> i32 { 0 }
//! #     fn do_vanish(&mut self) {}
//! # }
//! let parser = BulletMLParser::parse_file("pattern.xml").unwrap();
//! let mut runner = BulletMLRunner::from_parser(&parser);
//! let mut game = Game;
//! while !runner.is_end() {
//!     runner.run(&mut game); // call once per frame
//! }
//! ```

mod formula;
mod parser;
mod runner;
mod state;
mod tree;

pub use parser::{BulletMLParser, ParseError};
pub use runner::BulletMLRunner;
pub use state::BulletMLState;

/// The host-side interface the runner calls back into — the Rust equivalent of
/// subclassing the C++ `BulletMLRunner` and overriding its virtual methods.
///
/// The required methods report the controlled bullet's current state and enact
/// bullet creation / death. The provided methods cover the optional C++ virtuals
/// (`doChangeDirection`, `doAccel*`, `getBulletSpeedX/Y`, `getDefaultSpeed`,
/// `getRand`); override them as needed. All angles are degrees, clockwise from up.
pub trait AppRunner {
    /// Current heading of this bullet, in degrees (clockwise from straight up).
    fn get_bullet_direction(&mut self) -> f64;
    /// Direction from this bullet toward the aim target (e.g. the player), degrees.
    fn get_aim_direction(&mut self) -> f64;
    /// Current scalar speed of this bullet.
    fn get_bullet_speed(&mut self) -> f64;
    /// Difficulty rank, conventionally in `0.0..=1.0` (`$rank` in formulas).
    fn get_rank(&mut self) -> f64;
    /// Spawn a bullet with no `<action>` of its own (a plain moving bullet).
    fn create_simple_bullet(&mut self, direction: f64, speed: f64);
    /// Spawn a bullet that runs the given [`BulletMLState`]; pass the state to a
    /// new [`BulletMLRunner::from_state`].
    fn create_bullet(&mut self, state: BulletMLState, direction: f64, speed: f64);
    /// The current turn (frame counter); must be non-negative and share units
    /// with `<wait>`/`term`.
    fn get_turn(&mut self) -> i32;
    /// Destroy the controlled bullet (`<vanish>`).
    fn do_vanish(&mut self);

    /// Default speed used when a `<bullet>` specifies no `<speed>` (C++ default 1.0).
    fn get_default_speed(&mut self) -> f64 {
        1.0
    }
    /// A random number in `[0, 1)` for `$rand`. The C++ default uses `std::rand`;
    /// hosts that need determinism should override this.
    fn get_rand(&mut self) -> f64 {
        0.0
    }
    /// Set the bullet's heading to `direction` degrees (`<changeDirection>` step).
    fn do_change_direction(&mut self, _direction: f64) {}
    /// Set the bullet's speed (`<changeSpeed>` step).
    fn do_change_speed(&mut self, _speed: f64) {}
    /// Set the bullet's X velocity component (`<accel>` horizontal step).
    fn do_accel_x(&mut self, _accel: f64) {}
    /// Set the bullet's Y velocity component (`<accel>` vertical step).
    fn do_accel_y(&mut self, _accel: f64) {}
    /// Current X velocity component (override if `<accel>` is used).
    fn get_bullet_speed_x(&mut self) -> f64 {
        0.0
    }
    /// Current Y velocity component (override if `<accel>` is used).
    fn get_bullet_speed_y(&mut self) -> f64 {
        0.0
    }
}
