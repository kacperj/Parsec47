//! Port of src/abagames/p47/bullets/BulletActorPool.d — the bullet pool, the
//! BulletML callbacks the C++ engine calls back into, and the `bullets_*` API
//! consumed by the rest of the game (enemy / game_manager / ship).
use crate::actors::actor::Actor;
use crate::actors::actor_pool::ActorPool;
use crate::barrage::barrage_export::bulletml_register_callbacks;
use crate::bullets::bullet::Bullet;
use crate::bullets::bullet_actor::{
    reset_total_bullets_speed, take_ship_hit_events, total_bullets_speed, BulletActor,
};
use crate::bullets::ffi::*;
use crate::core::vector::Vector2;
use std::os::raw::c_void;

const POOL_SIZE: i32 = 512;

// BulletML "ShootDanmaku" <-> internal speed unit conversions (= BulletActorPool.d).
const VEL_SS_SDM_RATIO: f32 = 62.0 / 10.0;
const VEL_SDM_SS_RATIO: f32 = 10.0 / 62.0;

fn rtod(a: f32) -> f32 {
    a * 180.0 / std::f32::consts::PI
}

fn dtor(a: f32) -> f32 {
    a * std::f32::consts::PI / 180.0
}

pub struct BulletActorPool {
    pub pool: ActorPool<BulletActor>,
    pub cnt: i32,
    pub current_bullet: i32,
    pub target: Vector2,
}

impl BulletActorPool {
    fn new() -> Self {
        let mut pool = ActorPool::new(POOL_SIZE, BulletActor::new);
        // bullet.id == pool slot index, so doVanish(killMe) maps an id straight to a slot.
        for i in 0..pool.actors.len() {
            pool.actors[i].bullet.id = i as i32;
        }
        BulletActorPool {
            pool,
            cnt: 0,
            current_bullet: 0,
            target: Vector2 { x: 0.0, y: 0.0 },
        }
    }

    // addBullet(originalBullet, deg, speed) — spawn a child of the current bullet
    // (morph step if morphing, otherwise a plain simple bullet).
    fn add_simple_or_morph(&mut self, deg: f32, speed: f32) {
        let idx = match self.pool.get_instance_index() {
            Some(i) => i as usize,
            None => return,
        };
        let rb = self.pool.actors[self.current_bullet as usize].bullet;
        if rb.is_morph {
            let runner = unsafe { BulletMLRunner_new_parser(rb.morph_parser[rb.morph_idx as usize]) };
            regist_functions(runner);
            self.pool.actors[idx].set_runner_morph(
                runner, rb.pos.x, rb.pos.y, deg, speed, rb.rank, rb.speed_rank, rb.shape, rb.color,
                rb.bullet_size, rb.x_reverse, &rb.morph_parser, rb.morph_num, rb.morph_idx + 1,
                rb.morph_cnt - 1,
            );
        } else {
            self.pool.actors[idx].set_simple(
                rb.pos.x, rb.pos.y, deg, speed, rb.rank, rb.speed_rank, rb.shape, rb.color,
                rb.bullet_size, rb.x_reverse,
            );
        }
    }

    // addBullet(originalBullet, state, deg, speed) — spawn from a BulletML sub-state.
    fn add_from_state(&mut self, state: *mut c_void, deg: f32, speed: f32) {
        let idx = match self.pool.get_instance_index() {
            Some(i) => i as usize,
            None => return,
        };
        let runner = unsafe { BulletMLRunner_new_state(state) };
        regist_functions(runner);
        let rb = self.pool.actors[self.current_bullet as usize].bullet;
        if rb.is_morph {
            self.pool.actors[idx].set_runner_morph(
                runner, rb.pos.x, rb.pos.y, deg, speed, rb.rank, rb.speed_rank, rb.shape, rb.color,
                rb.bullet_size, rb.x_reverse, &rb.morph_parser, rb.morph_num, rb.morph_idx,
                rb.morph_cnt,
            );
        } else {
            self.pool.actors[idx].set_runner(
                runner, rb.pos.x, rb.pos.y, deg, speed, rb.rank, rb.speed_rank, rb.shape, rb.color,
                rb.bullet_size, rb.x_reverse,
            );
        }
    }

    fn kill_me(&mut self, bullet_id: i32) {
        self.pool.actors[bullet_id as usize].remove();
    }

    fn rewind(&mut self, i: usize) {
        self.pool.actors[i].bullet.remove();
        let parser = self.pool.actors[i].parser;
        let runner = unsafe { BulletMLRunner_new_parser(parser) };
        regist_functions(runner);
        self.pool.actors[i].bullet.set_runner(runner);
        self.pool.actors[i].bullet.reset_morph();
    }
}

// The reentrant per-frame step for one bullet. We never hold a borrow of the pool
// across BulletMLRunner_run, which synchronously calls the callbacks below and
// mutates the same pool; each `bullet_pool()` borrow is short-lived and dropped
// before the next, so the callbacks' borrows never overlap this one.
fn update_actor(i: usize) {
    {
        let a = &mut bullet_pool().pool.actors[i];
        a.ppos = a.bullet.pos;
    }
    let (is_simple, runner) = {
        let a = &bullet_pool().pool.actors[i];
        (a.is_simple, a.bullet.runner())
    };
    if !is_simple {
        if !runner.is_null() && unsafe { !BulletMLRunner_isEnd(runner) } {
            unsafe { BulletMLRunner_run(runner) };
        }
        let (is_top, r) = {
            let a = &bullet_pool().pool.actors[i];
            (a.is_top, a.bullet.runner())
        };
        if is_top && !r.is_null() && unsafe { BulletMLRunner_isEnd(r) } {
            bullet_pool().rewind(i);
        }
    }
    bullet_pool().pool.actors[i].advance();
}

// Registers every BulletML callback on a runner. The getDefaultSpeed/getRand pair
// is registered by the existing Rust helper; the rest are the callbacks below.
fn regist_functions(runner: *mut c_void) {
    unsafe {
        BulletMLRunner_set_getBulletDirection(runner, get_bullet_direction);
        BulletMLRunner_set_getAimDirection(runner, get_aim_direction_with_xrev);
        BulletMLRunner_set_getBulletSpeed(runner, get_bullet_speed);
        BulletMLRunner_set_getRank(runner, get_rank);
        BulletMLRunner_set_createSimpleBullet(runner, create_simple_bullet);
        BulletMLRunner_set_createBullet(runner, create_bullet);
        BulletMLRunner_set_getTurn(runner, get_turn);
        BulletMLRunner_set_doVanish(runner, do_vanish);
        BulletMLRunner_set_doChangeDirection(runner, do_change_direction);
        BulletMLRunner_set_doChangeSpeed(runner, do_change_speed);
        BulletMLRunner_set_doAccelX(runner, do_accel_x);
        BulletMLRunner_set_doAccelY(runner, do_accel_y);
        BulletMLRunner_set_getBulletSpeedX(runner, get_bullet_speed_x);
        BulletMLRunner_set_getBulletSpeedY(runner, get_bullet_speed_y);
    }
    // getDefaultSpeed and getRand are implemented and registered in Rust (barrage).
    bulletml_register_callbacks(runner);
}

// ---- Singleton --------------------------------------------------------------

static mut BULLET_POOL: Option<BulletActorPool> = None;

fn bullet_pool() -> &'static mut BulletActorPool {
    unsafe { BULLET_POOL.get_or_insert_with(BulletActorPool::new) }
}

// The bullet the BulletML engine is currently running (== pool.current_bullet).
fn cur(pool: &BulletActorPool) -> &Bullet {
    &pool.pool.actors[pool.current_bullet as usize].bullet
}

fn cur_mut(pool: &mut BulletActorPool) -> &mut Bullet {
    let ci = pool.current_bullet as usize;
    &mut pool.pool.actors[ci].bullet
}

// ---- BulletML callbacks (the C++ runner calls these back via fn pointers) ----

extern "C" fn get_bullet_direction(_r: *mut c_void) -> f64 {
    rtod(cur(bullet_pool()).deg) as f64
}

extern "C" fn get_bullet_speed(_r: *mut c_void) -> f64 {
    (cur(bullet_pool()).speed * VEL_SS_SDM_RATIO) as f64
}

extern "C" fn get_rank(_r: *mut c_void) -> f64 {
    cur(bullet_pool()).rank as f64
}

extern "C" fn create_simple_bullet(_r: *mut c_void, d: f64, s: f64) {
    bullet_pool().add_simple_or_morph(dtor(d as f32), s as f32 * VEL_SDM_SS_RATIO);
}

extern "C" fn create_bullet(_r: *mut c_void, state: *mut c_void, d: f64, s: f64) {
    bullet_pool().add_from_state(state, dtor(d as f32), s as f32 * VEL_SDM_SS_RATIO);
}

extern "C" fn get_turn(_r: *mut c_void) -> i32 {
    bullet_pool().cnt
}

extern "C" fn do_vanish(_r: *mut c_void) {
    let pool = bullet_pool();
    let id = cur(pool).id;
    pool.kill_me(id);
}

extern "C" fn do_change_direction(_r: *mut c_void, d: f64) {
    cur_mut(bullet_pool()).deg = dtor(d as f32);
}

extern "C" fn do_change_speed(_r: *mut c_void, s: f64) {
    cur_mut(bullet_pool()).speed = s as f32 * VEL_SDM_SS_RATIO;
}

extern "C" fn do_accel_x(_r: *mut c_void, sx: f64) {
    cur_mut(bullet_pool()).acc.x = sx as f32 * VEL_SDM_SS_RATIO;
}

extern "C" fn do_accel_y(_r: *mut c_void, sy: f64) {
    cur_mut(bullet_pool()).acc.y = sy as f32 * VEL_SDM_SS_RATIO;
}

extern "C" fn get_bullet_speed_x(_r: *mut c_void) -> f64 {
    cur(bullet_pool()).acc.x as f64
}

extern "C" fn get_bullet_speed_y(_r: *mut c_void) -> f64 {
    cur(bullet_pool()).acc.y as f64
}

extern "C" fn get_aim_direction_with_xrev(_r: *mut c_void) -> f64 {
    let pool = bullet_pool();
    let b = cur(pool).pos;
    let xrev = cur(pool).x_reverse;
    let dir = pool.target - b;
    rtod(dir.x.atan2(dir.y) * xrev) as f64
}

// ---- Public bullet API ------------------------------------------------------

/// Steps every bullet one frame. Returns the OR of ship-destruction event bits
/// (release-lock / destroyed) raised by bullet->ship collisions this frame, which
/// the game manager dispatches just as Ship.destroyed() used to.
pub fn bullets_update() -> i32 {
    let len = bullet_pool().pool.actors.len();
    for i in 0..len {
        let active = {
            let pool = bullet_pool();
            if pool.pool.actors[i].is_active() {
                if !pool.pool.actors[i].is_simple {
                    pool.current_bullet = i as i32;
                }
                true
            } else {
                false
            }
        };
        if active {
            update_actor(i);
        }
    }
    bullet_pool().cnt += 1;
    take_ship_hit_events()
}

pub fn bullets_draw() {
    bullet_pool().pool.draw();
}

pub fn bullets_clear() {
    for a in bullet_pool().pool.actors.iter_mut() {
        if a.is_active() {
            a.remove();
        }
    }
}

pub fn bullets_to_retro_all() {
    for a in bullet_pool().pool.actors.iter_mut() {
        if a.is_active() {
            a.to_retro();
        }
    }
}

pub fn bullets_reset_total_speed() {
    reset_total_bullets_speed();
}

pub fn bullets_get_total_speed() -> f32 {
    total_bullets_speed()
}

pub fn bullets_set_target(x: f32, y: f32) {
    bullet_pool().target = Vector2 { x, y };
}

/// Invisible BulletML-driven bullet (the move bullet). Owns runner creation and
/// callback registration; the caller passes only the parser. Returns slot index or -1.
#[allow(clippy::too_many_arguments)]
pub fn bullets_add(
    parser: *mut c_void,
    x: f32,
    y: f32,
    deg: f32,
    speed: f32,
    rank: f32,
    speed_rank: f32,
    shape: i32,
    color: i32,
    size: f32,
    x_reverse: f32,
) -> i32 {
    let pool = bullet_pool();
    let idx = match pool.pool.get_instance_index() {
        Some(i) => i,
        None => return -1,
    };
    let runner = unsafe { BulletMLRunner_new_parser(parser) };
    regist_functions(runner);
    let a = &mut pool.pool.actors[idx as usize];
    a.set_runner(runner, x, y, deg, speed, rank, speed_rank, shape, color, size, x_reverse);
    a.set_invisible();
    idx
}

/// Top-level pattern bullet (no morph).
#[allow(clippy::too_many_arguments)]
pub fn bullets_add_top(
    parser: *mut c_void,
    x: f32,
    y: f32,
    deg: f32,
    speed: f32,
    rank: f32,
    speed_rank: f32,
    shape: i32,
    color: i32,
    size: f32,
    x_reverse: f32,
) -> i32 {
    let idx = bullets_add(parser, x, y, deg, speed, rank, speed_rank, shape, color, size, x_reverse);
    if idx < 0 {
        return -1;
    }
    bullet_pool().pool.actors[idx as usize].set_top(parser);
    idx
}

/// Top-level pattern bullet with a morph chain. `morph` points to an array of
/// `morph_num` BulletMLParser* (the caller passes `br.morph_parser.as_ptr()`).
#[allow(clippy::too_many_arguments)]
pub fn bullets_add_top_morph(
    parser: *mut c_void,
    x: f32,
    y: f32,
    deg: f32,
    speed: f32,
    rank: f32,
    speed_rank: f32,
    shape: i32,
    color: i32,
    size: f32,
    x_reverse: f32,
    morph: *const *mut c_void,
    morph_num: i32,
    morph_cnt: i32,
) -> i32 {
    let pool = bullet_pool();
    let idx = match pool.pool.get_instance_index() {
        Some(i) => i,
        None => return -1,
    };
    let runner = unsafe { BulletMLRunner_new_parser(parser) };
    regist_functions(runner);
    let morph_slice = unsafe { std::slice::from_raw_parts(morph, morph_num as usize) };
    let a = &mut pool.pool.actors[idx as usize];
    a.set_runner_morph(
        runner, x, y, deg, speed, rank, speed_rank, shape, color, size, x_reverse, morph_slice,
        morph_num, 0, morph_cnt,
    );
    a.set_top(parser);
    idx
}

fn valid_index(i: i32) -> Option<usize> {
    if i < 0 || i as usize >= bullet_pool().pool.actors.len() {
        None
    } else {
        Some(i as usize)
    }
}

pub fn bullets_get_deg(i: i32) -> f32 {
    match valid_index(i) {
        Some(i) => bullet_pool().pool.actors[i].bullet.deg,
        None => 0.0,
    }
}

pub fn bullets_get_pos_x(i: i32) -> f32 {
    match valid_index(i) {
        Some(i) => bullet_pool().pool.actors[i].bullet.pos.x,
        None => 0.0,
    }
}

pub fn bullets_get_pos_y(i: i32) -> f32 {
    match valid_index(i) {
        Some(i) => bullet_pool().pool.actors[i].bullet.pos.y,
        None => 0.0,
    }
}

pub fn bullets_set_pos(i: i32, x: f32, y: f32) {
    if let Some(i) = valid_index(i) {
        bullet_pool().pool.actors[i].bullet.pos = Vector2 { x, y };
    }
}

pub fn bullets_remove(i: i32) {
    if let Some(i) = valid_index(i) {
        bullet_pool().pool.actors[i].remove();
    }
}
