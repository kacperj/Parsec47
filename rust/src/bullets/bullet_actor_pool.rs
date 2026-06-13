//! Port of src/abagames/p47/bullets/BulletActorPool.d — the bullet pool, the
//! host callbacks the `bulletml` runner invokes (the `GameRunner` AppRunner impl),
//! and the `bullets_*` API consumed by the rest of the game (enemy / game_manager / ship).
use crate::actors::actor::Actor;
use crate::actors::actor_pool::ActorPool;
use crate::bullets::bullet::Bullet;
use crate::bullets::bullet_actor::{
    reset_total_bullets_speed, take_ship_hit_events, total_bullets_speed, BulletActor,
};
use crate::core::rand::genrand_real1;
use crate::core::vector::Vector2;
use bulletml::{AppRunner, BulletMLParser, BulletMLRunner, BulletMLState};
use std::os::raw::c_void;

const POOL_SIZE: i32 = 512;

// Runners and parsers travel through the pool as opaque `*mut c_void` (so the
// `Copy` bullet structs stay pointer-sized). These helpers box/borrow/reclaim
// the real `bulletml` types at the boundary.

/// Box a fresh runner for `parser` and return it as an opaque handle.
fn new_runner_from_parser(parser: *mut c_void) -> *mut c_void {
    // SAFETY: parsers are boxed in BarrageManager::load_bulletmls and outlive
    // every runner created from them.
    let p = unsafe { &*(parser as *const BulletMLParser) };
    Box::into_raw(Box::new(BulletMLRunner::from_parser(p))) as *mut c_void
}

/// Box a runner resuming a BulletML sub-state, returned as an opaque handle.
fn new_runner_from_state(state: BulletMLState) -> *mut c_void {
    Box::into_raw(Box::new(BulletMLRunner::from_state(state))) as *mut c_void
}

/// Step a runner one frame, driving it with the game's `GameRunner` host.
fn runner_run(runner: *mut c_void) {
    // SAFETY: `runner` is a live boxed BulletMLRunner; the GameRunner host reaches
    // the bullet pool through the global singleton, never through this borrow, so
    // child bullets spawned mid-run never alias `*runner`.
    unsafe { (*(runner as *mut BulletMLRunner)).run(&mut GameRunner) };
}

fn runner_is_end(runner: *mut c_void) -> bool {
    unsafe { (*(runner as *mut BulletMLRunner)).is_end() }
}

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
            let runner = new_runner_from_parser(rb.morph_parser[rb.morph_idx as usize]);
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
    fn add_from_state(&mut self, state: BulletMLState, deg: f32, speed: f32) {
        let idx = match self.pool.get_instance_index() {
            Some(i) => i as usize,
            None => return,
        };
        let runner = new_runner_from_state(state);
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
        let runner = new_runner_from_parser(parser);
        self.pool.actors[i].bullet.set_runner(runner);
        self.pool.actors[i].bullet.reset_morph();
    }
}

// The reentrant per-frame step for one bullet. We never hold a borrow of the pool
// across the runner's run(), which synchronously calls the GameRunner host below
// and mutates the same pool; each `bullet_pool()` borrow is short-lived and dropped
// before the next, so the host's borrows never overlap this one.
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
        if !runner.is_null() && !runner_is_end(runner) {
            runner_run(runner);
        }
        let (is_top, r) = {
            let a = &bullet_pool().pool.actors[i];
            (a.is_top, a.bullet.runner())
        };
        if is_top && !r.is_null() && runner_is_end(r) {
            bullet_pool().rewind(i);
        }
    }
    bullet_pool().pool.actors[i].advance();
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

// ---- BulletML host (the `bulletml` runner calls these back via the trait) ----

// A zero-sized host: every method operates on the global bullet pool and the
// pool's `current_bullet` (the bullet the runner is stepping), exactly as the old
// C-ABI callbacks did. Reentrant pool mutation is safe because this holds no
// borrow of its own.
struct GameRunner;

impl AppRunner for GameRunner {
    fn get_bullet_direction(&mut self) -> f64 {
        rtod(cur(bullet_pool()).deg) as f64
    }

    fn get_aim_direction(&mut self) -> f64 {
        let pool = bullet_pool();
        let b = cur(pool).pos;
        let xrev = cur(pool).x_reverse;
        let dir = pool.target - b;
        rtod(dir.x.atan2(dir.y) * xrev) as f64
    }

    fn get_bullet_speed(&mut self) -> f64 {
        (cur(bullet_pool()).speed * VEL_SS_SDM_RATIO) as f64
    }

    fn get_rank(&mut self) -> f64 {
        cur(bullet_pool()).rank as f64
    }

    fn create_simple_bullet(&mut self, direction: f64, speed: f64) {
        bullet_pool().add_simple_or_morph(dtor(direction as f32), speed as f32 * VEL_SDM_SS_RATIO);
    }

    fn create_bullet(&mut self, state: BulletMLState, direction: f64, speed: f64) {
        bullet_pool().add_from_state(state, dtor(direction as f32), speed as f32 * VEL_SDM_SS_RATIO);
    }

    fn get_turn(&mut self) -> i32 {
        bullet_pool().cnt
    }

    fn do_vanish(&mut self) {
        let pool = bullet_pool();
        let id = cur(pool).id;
        pool.kill_me(id);
    }

    fn get_default_speed(&mut self) -> f64 {
        1.0
    }

    fn get_rand(&mut self) -> f64 {
        genrand_real1()
    }

    fn do_change_direction(&mut self, direction: f64) {
        cur_mut(bullet_pool()).deg = dtor(direction as f32);
    }

    fn do_change_speed(&mut self, speed: f64) {
        cur_mut(bullet_pool()).speed = speed as f32 * VEL_SDM_SS_RATIO;
    }

    fn do_accel_x(&mut self, accel: f64) {
        cur_mut(bullet_pool()).acc.x = accel as f32 * VEL_SDM_SS_RATIO;
    }

    fn do_accel_y(&mut self, accel: f64) {
        cur_mut(bullet_pool()).acc.y = accel as f32 * VEL_SDM_SS_RATIO;
    }

    fn get_bullet_speed_x(&mut self) -> f64 {
        cur(bullet_pool()).acc.x as f64
    }

    fn get_bullet_speed_y(&mut self) -> f64 {
        cur(bullet_pool()).acc.y as f64
    }
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
    let runner = new_runner_from_parser(parser);
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
    let runner = new_runner_from_parser(parser);
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
