//! Port of src/abagames/p47/bullets/BulletActor.d — wraps a Bullet with actor-pool
//! semantics, ship collision, the retro fade-in/out, and rendering.
use crate::actors::actor::Actor;
use crate::bullets::bullet::Bullet;
use crate::core::vector::Vector2;
use crate::field::field_check_hit_with_space;
use crate::ship::{ship_destroyed, ship_get_cnt, ship_get_pos_x, ship_get_pos_y};
use std::os::raw::c_void;
use std::ptr;

const FIELD_SPACE: f32 = 0.5;
const BULLET_DISAPPEAR_CNT: i32 = 180;
const SHIP_HIT_WIDTH: f32 = 0.2;
const RETRO_CNT: f32 = 24.0;
const INVINCIBLE_CNT: i32 = 228; // = Ship.INVINCIBLE_CNT

// Class-level accumulator (= BulletActor.totalBulletsSpeed). Reset each frame by
// the game manager via bullets_reset_total_speed and read back for slowdown.
static mut TOTAL_BULLETS_SPEED: f32 = 0.0;

// Ship-destruction events raised by bullet collisions during one bullets_update
// pass. Drained (returned and cleared) by bullets_update so the game manager
// can run shipDestroyed()/releaseLock(), exactly as D's Ship.destroyed() did.
static mut SHIP_HIT_EVENTS: i32 = 0;

pub fn reset_total_bullets_speed() {
    unsafe { TOTAL_BULLETS_SPEED = 0.0 };
}

pub fn total_bullets_speed() -> f32 {
    unsafe { TOTAL_BULLETS_SPEED }
}

pub fn take_ship_hit_events() -> i32 {
    unsafe {
        let e = SHIP_HIT_EVENTS;
        SHIP_HIT_EVENTS = 0;
        e
    }
}

pub struct BulletActor {
    pub bullet: Bullet,
    pub is_simple: bool,
    pub is_top: bool,
    pub is_visible: bool,
    pub parser: *mut c_void,
    pub ppos: Vector2,
    pub cnt: i32,
    pub rt_cnt: f32,
    pub should_be_removed: bool,
    pub back_to_retro: bool,
    pub is_exist: bool,
}

impl BulletActor {
    pub fn new() -> Self {
        BulletActor {
            bullet: Bullet::new(0),
            is_simple: false,
            is_top: false,
            is_visible: false,
            parser: ptr::null_mut(),
            ppos: Vector2 { x: 0.0, y: 0.0 },
            cnt: 0,
            rt_cnt: 0.0,
            should_be_removed: false,
            back_to_retro: false,
            is_exist: false,
        }
    }

    fn start(&mut self, speed_rank: f32, shape: i32, color: i32, size: f32, x_reverse: f32) {
        self.is_exist = true;
        self.is_top = false;
        self.is_visible = true;
        self.ppos = self.bullet.pos;
        self.bullet.set_param(speed_rank, shape, color, size, x_reverse);
        self.cnt = 0;
        self.rt_cnt = 0.0;
        self.should_be_removed = false;
        self.back_to_retro = false;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_runner(
        &mut self,
        runner: *mut c_void,
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
    ) {
        self.bullet.set(x, y, deg, speed, rank);
        self.bullet.set_runner(runner);
        self.bullet.is_morph = false;
        self.is_simple = false;
        self.start(speed_rank, shape, color, size, x_reverse);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_runner_morph(
        &mut self,
        runner: *mut c_void,
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
        morph: &[*mut c_void],
        morph_num: i32,
        morph_idx: i32,
        morph_cnt: i32,
    ) {
        self.bullet.set(x, y, deg, speed, rank);
        self.bullet.set_runner(runner);
        self.bullet.set_morph(morph, morph_num, morph_idx, morph_cnt);
        self.is_simple = false;
        self.start(speed_rank, shape, color, size, x_reverse);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_simple(
        &mut self,
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
    ) {
        self.bullet.set(x, y, deg, speed, rank);
        self.bullet.is_morph = false;
        self.is_simple = true;
        self.start(speed_rank, shape, color, size, x_reverse);
    }

    pub fn set_invisible(&mut self) {
        self.is_visible = false;
    }

    pub fn set_top(&mut self, parser: *mut c_void) {
        self.parser = parser;
        self.is_top = true;
        self.set_invisible();
    }

    pub fn remove(&mut self) {
        self.should_be_removed = true;
    }

    fn remove_forced(&mut self) {
        if !self.is_simple {
            self.bullet.remove();
        }
        self.is_exist = false;
    }

    pub fn to_retro(&mut self) {
        if !self.is_visible || self.back_to_retro {
            return;
        }
        self.back_to_retro = true;
        if self.rt_cnt >= RETRO_CNT {
            self.rt_cnt = RETRO_CNT - 0.1;
        }
    }

    // Swept-segment collision of the bullet (ppos -> pos) against the ship point.
    fn check_ship_hit(&mut self) {
        let bmvx = self.ppos.x - self.bullet.pos.x;
        let bmvy = self.ppos.y - self.bullet.pos.y;
        let inaa = bmvx * bmvx + bmvy * bmvy;
        if inaa > 0.00001 {
            let sofsx = ship_get_pos_x() - self.bullet.pos.x;
            let sofsy = ship_get_pos_y() - self.bullet.pos.y;
            let inab = bmvx * sofsx + bmvy * sofsy;
            if inab >= 0.0 && inab <= inaa {
                let hd = sofsx * sofsx + sofsy * sofsy - inab * inab / inaa;
                if hd >= 0.0 && hd <= SHIP_HIT_WIDTH {
                    unsafe { SHIP_HIT_EVENTS |= ship_destroyed() };
                }
            }
        }
    }

    // The non-reentrant tail of BulletActor.move(): runs after BulletMLRunner_run
    // has executed (and any rewind). Returns with is_exist=false if the bullet died.
    pub fn advance(&mut self) {
        if self.should_be_removed {
            self.remove_forced();
            return;
        }
        let sr;
        if self.rt_cnt < RETRO_CNT {
            sr = self.bullet.speed_rank * (0.3 + (self.rt_cnt / RETRO_CNT) * 0.7);
            if self.back_to_retro {
                self.rt_cnt -= sr;
                if self.rt_cnt <= 0.0 {
                    self.remove_forced();
                    return;
                }
            } else {
                self.rt_cnt += sr;
            }
            if ship_get_cnt() < -INVINCIBLE_CNT / 2 && self.is_visible && self.rt_cnt >= RETRO_CNT {
                self.remove_forced();
                return;
            }
        } else {
            sr = self.bullet.speed_rank;
            if self.cnt > BULLET_DISAPPEAR_CNT {
                self.to_retro();
            }
        }
        self.bullet.pos.x +=
            (self.bullet.deg.sin() * self.bullet.speed + self.bullet.acc.x) * sr * self.bullet.x_reverse;
        self.bullet.pos.y += (self.bullet.deg.cos() * self.bullet.speed - self.bullet.acc.y) * sr;
        if self.is_visible {
            unsafe { TOTAL_BULLETS_SPEED += self.bullet.speed * sr };
            if self.rt_cnt > RETRO_CNT {
                self.check_ship_hit();
            }
            if field_check_hit_with_space(self.bullet.pos.x, self.bullet.pos.y, FIELD_SPACE) {
                self.remove_forced();
                return;
            }
        }
        self.cnt += 1;
    }
}

impl Actor for BulletActor {
    // The per-frame step is driven by BulletActorPool::update (raw-pointer access)
    // because BulletML callbacks reenter the pool mid-run; this trait method is
    // intentionally unused.
    fn update(&mut self) {}

    fn draw(&self) {
        if !self.is_visible {
            return;
        }
        crate::bullet_actor::bullet_actor_draw(
            self.bullet.shape,
            self.bullet.color,
            self.bullet.deg,
            self.bullet.x_reverse,
            self.cnt,
            self.bullet.pos.x,
            self.bullet.pos.y,
            self.rt_cnt,
            self.bullet.bullet_size,
        );
    }

    fn draw_luminous(&self) {}

    fn is_active(&self) -> bool {
        self.is_exist
    }

    fn set_active(&mut self, active: bool) {
        self.is_exist = active;
    }
}
