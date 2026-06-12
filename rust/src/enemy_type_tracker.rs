//! Tracks which enemy types are currently alive on the field. Repopulated each
//! frame by the enemy pool's move (enemy.rs). Formerly EnemyTypeTracker.d, now
//! fully owned here; the enemy pool and StageManager share this one table.

const ENEMY_TYPE_MAX: usize = 32;

static mut TYPES: [bool; ENEMY_TYPE_MAX] = [false; ENEMY_TYPE_MAX];

pub fn mark(id: i32) {
    unsafe {
        TYPES[id as usize] = true;
    }
}

pub fn exists(id: i32) -> bool {
    unsafe { TYPES[id as usize] }
}

pub fn clear() {
    unsafe {
        TYPES = [false; ENEMY_TYPE_MAX];
    }
}
