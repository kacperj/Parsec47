use std::os::raw::c_void;

use crate::barrage::{Barrage, BarrageManager};

static mut BARRAGE_MANAGER: BarrageManager = BarrageManager::new();

pub fn barrage_load_bulletmls() {
    unsafe {
        BARRAGE_MANAGER.load_bulletmls();
    }
}

pub fn barrage_get_move_parser(category: i32, move_type_random: i32) -> *mut c_void {
    unsafe { BARRAGE_MANAGER.get_move_parser(category, move_type_random) }
}

// EnemyType (rust/src/enemy_type.rs) is the only caller.
pub fn barrage_create(btn: i32, mode: i32) -> Barrage {
    unsafe { BARRAGE_MANAGER.create_barrage(btn, mode) }
}

pub fn barrage_unload_bulletmls() {
    unsafe {
        BARRAGE_MANAGER.unload_bulletmls();
    }
}
