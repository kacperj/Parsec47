use core::ffi::c_int;

use crate::pad::{
    is_any_direction_pressed, is_down_pressed, is_left_pressed, is_right_pressed, is_up_pressed,
};
use crate::prefs::prefs_get_slots;
use crate::ui_renderer::renderer_draw_title;

const DIFFICULTY_NUM: c_int = 4;
const MODE_NUM: c_int = 2;
const VERTICAL_BOXES_COUNT: c_int = DIFFICULTY_NUM + 1;
const BOX_COUNT: c_int = 16;

static mut CUR_X: c_int = 0;
static mut CUR_Y: c_int = 0;
static mut MODE: c_int = 0;
static mut BOX_CNT: c_int = 0;
static mut STAGE_CHANGED: bool = false;
static mut PAD_PRSD: bool = true;

fn get_slots(difficulty: c_int) -> c_int {
    let mode = unsafe { MODE };
    prefs_get_slots(mode, difficulty)
}

pub fn title_start(difficulty: c_int, parsec_slot: c_int, mode: c_int) {
    unsafe {
        CUR_X = parsec_slot;
        CUR_Y = difficulty;
        MODE = mode;
        BOX_CNT = BOX_COUNT;
    }
}

pub fn title_move() {
    unsafe {
        STAGE_CHANGED = false;

        let up = is_up_pressed();
        let down = is_down_pressed();
        let left = is_left_pressed();
        let right = is_right_pressed();
        let any_dir = is_any_direction_pressed();

        if !PAD_PRSD {
            if down {
                CUR_Y += 1;
            } else if up {
                CUR_Y -= 1;
            } else if right {
                CUR_X += 1;
                let slots = get_slots(CUR_Y);
                CUR_X = (CUR_X + slots) % slots;
            } else if left {
                CUR_X -= 1;
                let slots = get_slots(CUR_Y);
                CUR_X = (CUR_X + slots) % slots;
            }

            CUR_Y = (CUR_Y + VERTICAL_BOXES_COUNT) % VERTICAL_BOXES_COUNT;
            let slots = get_slots(CUR_Y);
            if CUR_X >= slots {
                CUR_X = slots - 1;
            }

            if any_dir {
                BOX_CNT = BOX_COUNT;
                PAD_PRSD = true;
                STAGE_CHANGED = true;
            }
        } else if !any_dir {
            PAD_PRSD = false;
        }

        if BOX_CNT >= 0 {
            BOX_CNT -= 1;
        }
    }
}

pub fn title_should_change_stage() -> c_int {
    unsafe {
        if STAGE_CHANGED {
            1
        } else {
            0
        }
    }
}

pub fn title_get_cur_x() -> c_int {
    unsafe { CUR_X }
}

pub fn title_get_cur_y() -> c_int {
    unsafe { CUR_Y }
}

pub fn title_get_mode() -> c_int {
    unsafe { MODE }
}

pub fn title_change_mode() {
    unsafe {
        MODE += 1;
        if MODE >= MODE_NUM {
            MODE = 0;
        }
        let slots = get_slots(CUR_Y);
        if CUR_X >= slots {
            CUR_X = slots - 1;
        }
    }
}

pub fn title_draw() {
    unsafe {
        renderer_draw_title(CUR_X, CUR_Y, MODE, BOX_CNT);
    }
}
