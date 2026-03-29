use crate::state::score_state::ScoreState;

static mut SCORE_STATE: ScoreState = ScoreState::new();

#[no_mangle]
pub extern "C" fn get_bonus_state() -> i32 {
    unsafe { SCORE_STATE.get_bonus_score() }
}

#[no_mangle]
pub extern "C" fn bonus_state_reset() {
    unsafe {
        SCORE_STATE.reset_bonus_score();
    }
}

#[no_mangle]
pub extern "C" fn score_set_initial() {
    unsafe {
        SCORE_STATE.set_initial();
    }
}

#[no_mangle]
pub extern "C" fn life_get() -> i32 {
    unsafe { SCORE_STATE.get_life() }
}

#[no_mangle]
pub extern "C" fn life_decrease() {
    unsafe {
        SCORE_STATE.decrease_life();
    }
}

#[no_mangle]
pub extern "C" fn score_get() -> i32 {
    unsafe { SCORE_STATE.get_score() }
}

#[no_mangle]
pub extern "C" fn score_increase(sc: i32) {
    unsafe {
        SCORE_STATE.increase_score(sc);
    }
}

#[no_mangle]
pub extern "C" fn bonus_collected() {
    unsafe {
        SCORE_STATE.bonus_collected();
    }
}