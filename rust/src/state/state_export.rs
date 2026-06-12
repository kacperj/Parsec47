use crate::state::score_state::ScoreState;
use std::ptr::addr_of_mut;

static mut SCORE_STATE: ScoreState = ScoreState::new();

/// Accessor for the global score/life state. Single instance; the game runs on
/// one thread (the SDL main loop), so handing out a `&'static mut` is sound.
pub fn score_state() -> &'static mut ScoreState {
    unsafe { &mut *addr_of_mut!(SCORE_STATE) }
}
