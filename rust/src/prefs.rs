use serde::{Deserialize, Serialize};
use std::fs;

const PREFS_FILE: &str = "p47.json";

const MODE_NUM: usize = 2;
const DIFFICULTY_NUM: usize = 4;
const SLOT_NUM: usize = 10;

#[derive(Serialize, Deserialize)]
pub struct ProgressData {
    pub hi_score: [[[i32; SLOT_NUM]; DIFFICULTY_NUM]; MODE_NUM],
    pub reached_parsec: [[i32; DIFFICULTY_NUM]; MODE_NUM],
    pub selected_difficulty: i32,
    pub selected_parsec_slot: i32,
    pub selected_mode: i32,
}

impl Default for ProgressData {
    fn default() -> Self {
        ProgressData {
            hi_score: [[[0; SLOT_NUM]; DIFFICULTY_NUM]; MODE_NUM],
            reached_parsec: [[0; DIFFICULTY_NUM]; MODE_NUM],
            selected_difficulty: 1,
            selected_parsec_slot: 0,
            selected_mode: 0,
        }
    }
}

static mut DATA: Option<ProgressData> = None;

fn data() -> &'static mut ProgressData {
    unsafe {
        if DATA.is_none() {
            DATA = Some(ProgressData::default());
        }
        DATA.as_mut().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn prefs_load() {
    let loaded = fs::read_to_string(PREFS_FILE)
        .ok()
        .and_then(|s| serde_json::from_str::<ProgressData>(&s).ok());

    let d = match loaded {
        Some(parsed) => parsed,
        None => {
            let defaults = ProgressData::default();
            // Write the default file so the user has a readable starting point
            if let Ok(json) = serde_json::to_string_pretty(&defaults) {
                let _ = fs::write(PREFS_FILE, json);
            }
            defaults
        }
    };

    unsafe {
        DATA = Some(d);
    }
}

#[no_mangle]
pub extern "C" fn prefs_save() {
    let d = data();
    if let Ok(json) = serde_json::to_string_pretty(d) {
        let _ = fs::write(PREFS_FILE, json);
    }
}

#[no_mangle]
pub extern "C" fn prefs_hi_score_ptr() -> *mut i32 {
    &raw mut data().hi_score as *mut i32
}

#[no_mangle]
pub extern "C" fn prefs_reached_parsec_ptr() -> *mut i32 {
    &raw mut data().reached_parsec as *mut i32
}

#[no_mangle]
pub extern "C" fn prefs_get_selected_difficulty() -> i32 {
    data().selected_difficulty
}

#[no_mangle]
pub extern "C" fn prefs_set_selected_difficulty(val: i32) {
    data().selected_difficulty = val;
}

#[no_mangle]
pub extern "C" fn prefs_get_selected_parsec_slot() -> i32 {
    data().selected_parsec_slot
}

#[no_mangle]
pub extern "C" fn prefs_set_selected_parsec_slot(val: i32) {
    data().selected_parsec_slot = val;
}

#[no_mangle]
pub extern "C" fn prefs_get_selected_mode() -> i32 {
    data().selected_mode
}

#[no_mangle]
pub extern "C" fn prefs_set_selected_mode(val: i32) {
    data().selected_mode = val;
}
