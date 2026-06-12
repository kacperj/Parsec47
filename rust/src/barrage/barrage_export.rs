use std::os::raw::c_void;

use crate::barrage::{
    Barrage, BarrageManager, BulletMLRunner_set_getDefaultSpeed, BulletMLRunner_set_getRand,
};
use crate::core::rand::genrand_real1;

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

// BulletML runner callbacks, registered on the runner by
// bulletml_register_callbacks below. The remaining callbacks live in
// bullet_actor_pool.rs and are registered by its regist_functions.

extern "C" fn bulletml_get_rand(_r: *mut c_void) -> f64 {
    genrand_real1()
}

extern "C" fn bulletml_get_default_speed(_r: *mut c_void) -> f64 {
    1.0
}

/// Registers the Rust-implemented BulletML callbacks on a runner.
/// Called from bullet_actor_pool::regist_functions.
pub fn bulletml_register_callbacks(runner: *mut c_void) {
    unsafe {
        BulletMLRunner_set_getDefaultSpeed(runner, bulletml_get_default_speed);
        BulletMLRunner_set_getRand(runner, bulletml_get_rand);
    }
}
