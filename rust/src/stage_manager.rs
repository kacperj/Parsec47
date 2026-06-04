use crate::core::rand::rand_next_int;
use core::ffi::c_int;

const MIDDLE_RUSH_SECTION_PATTERN: c_int = 6;

// [#smalltype, #middletype, #largetype] per section pattern, indexed by mode.
const APPEARANCE_PATTERN: [[[c_int; 3]; 7]; 2] = [
    // ROLL
    [
        [1, 0, 0],
        [2, 0, 0],
        [1, 1, 0],
        [1, 0, 1],
        [2, 1, 0],
        [2, 0, 1],
        [0, 1, 1],
    ],
    // LOCK
    [
        [1, 0, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 0, 1],
        [2, 1, 0],
        [1, 1, 1],
        [0, 1, 1],
    ],
];

fn get_appearance_for_section(section: c_int, middle_rush_section_num: c_int) -> c_int {
    if section == 0 {
        return 0;
    }
    if section == middle_rush_section_num {
        return MIDDLE_RUSH_SECTION_PATTERN;
    }
    let sp = section * 3 / 7 + 1;
    let ep = 3 + section * 3 / 10;
    sp + rand_next_int(ep - sp + 1)
}

#[no_mangle]
pub extern "C" fn stage_get_appearance_count_for_section(
    section: c_int,
    middle_rush_section_num: c_int,
    mode: c_int,
    enemy_type: c_int,
) -> c_int {
    let ap = get_appearance_for_section(section, middle_rush_section_num);
    APPEARANCE_PATTERN[mode as usize][ap as usize][enemy_type as usize]
}
