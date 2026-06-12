//! Port of Enemy.d: the enemy actor (regular + boss), its pool, and the API
//! the game manager / stage manager call. Owns enemy movement, firing
//! (top-level BulletML bullets), collision against shots/rolls/locks, fragment
//! and bonus spawning, and the lock->enemy target association.
use crate::actors::actor::Actor;
use crate::actors::actor_export::{
    lock_get_laser_head_x, lock_get_laser_head_y, lock_get_lock_min_y, lock_get_state, lock_hit,
    lock_set_lock_min_y, lock_set_state, lock_set_target_snapshot, locks_is_active,
    locks_pool_size, particles_init_new, rolls_get_cnt, rolls_get_pos0_x, rolls_get_pos0_y,
    rolls_is_active, rolls_is_released, rolls_pool_size, shots_get_pos_x, shots_get_pos_y,
    shots_is_active, shots_set_inactive,
};
use crate::actors::actor_pool::ActorPool;
use crate::actors::actor_export::fragments_init_new;
use crate::actors::bonus::{bonus_get_rate, bonuses_add};
use crate::barrage::barrage_export::barrage_get_move_parser;
use crate::barrage::Barrage;
use crate::bullets::bullet_actor_pool::{
    bullets_add, bullets_add_top, bullets_add_top_morph, bullets_get_deg, bullets_get_pos_x,
    bullets_get_pos_y, bullets_remove, bullets_set_pos, bullets_to_retro_all,
};
use crate::enemy_type::{
    BatteryType, EnemyType, BATTERY_MAX, BODY_SHAPE_POINT_NUM, TYPE_SMALL, WING_BATTERY_MAX,
    WING_SHAPE_POINT_NUM,
};
use crate::field::{field_check_hit, field_get_collision_box};
use crate::game_manager;
use crate::enemy_type_tracker;
use crate::core::rand::{rand_next_float, rand_next_int, rand_next_signed_float};
use crate::core::vector::Vector2;
use crate::renderer::{draw_line_retro_with_z, set_color};
use crate::rendering::color::Color;
use crate::rendering::gl::{glBegin, glEnd, glVertex3f, GL_TRIANGLE_FAN};
use crate::screen_shake::screen_shake_set;
use crate::sound::sound_manager_play_se;
use crate::state::state_export::score_state;
use core::ffi::{c_float, c_int, c_void};
use std::f32::consts::PI;

// = Enemy.d
const ENEMY_MAX: i32 = 32;
const MOVE_POINT_MAX: usize = 8;
const ROLL_NO_COLLISION_CNT: i32 = 45;
const SHOT_SPEED: f32 = 1.0;
const APPEARANCE_CNT: i32 = 90;
const APPEARANCE_Z: f32 = -15.0;
const DESTROYED_CNT: i32 = 90;
const DESTROYED_Z: f32 = -10.0;
const TIMEOUT_CNT: i32 = 90;
const BOSS_TIMEOUT: i32 = 30 * 60;
const BOSS_MOVE_DEG: f32 = 0.02;

const SHOT_DAMAGE: i32 = 1;
const ROLL_DAMAGE: i32 = 1;
const LOCK_DAMAGE: i32 = 7;
const ENEMY_TYPE_SCORE: [i32; 5] = [100, 500, 1000, 5000, 10000];
const ENEMY_WING_SCORE: i32 = 1000;

// checkHit / checkLocked sentinels (mirror the enum in Enemy.d). Battery part
// hits return the battery index (>= 0).
const NOHIT: i32 = -2;
const HIT: i32 = -1;

// Lock states (mirror the enum in lock.rs / Lock.d).
const LOCK_SEARCH: i32 = 0;
const LOCK_SEARCHED: i32 = 1;
const LOCK_FIRED: i32 = 4;

const LOCK_NUM: usize = 4;

// Ship mode (= ShipMode.d). Only ROLL needs distinguishing here.
const MODE_ROLL: c_int = 0;

// SE indices (mirror SoundManager.d enum).
const SE_ENEMY_DESTROYED: c_int = 6;
const SE_LARGE_ENEMY_DESTROYED: c_int = 7;
const SE_BOSS_DESTROYED: c_int = 8;

// The retro parameter is always fully retro in the enemy draw (= RetroParam.retro).
const RETRO: c_float = 1.0;

const VEC_ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

fn field_half() -> (f32, f32) {
    let b = field_get_collision_box();
    ((b.x2 - b.x1) * 0.5, (b.y2 - b.y1) * 0.5)
}

/// Side wing with batteries (= Enemy.Battery in Enemy.d).
#[derive(Copy, Clone)]
struct Battery {
    // Bullet pool handles (-1 = none).
    top_bullet: [i32; WING_BATTERY_MAX],
    shield: i32,
    damaged: bool,
}

impl Battery {
    const fn new() -> Self {
        Battery {
            top_bullet: [-1; WING_BATTERY_MAX],
            shield: 0,
            damaged: false,
        }
    }
}

pub struct Enemy {
    active: bool, // = isExist
    pos: Vector2,
    ty: EnemyType, // = type
    battery: [Battery; BATTERY_MAX],
    shield: i32,
    cnt: i32,
    top_bullet: i32,
    move_bullet: i32,
    move_point: [Vector2; MOVE_POINT_MAX],
    move_point_num: i32,
    move_point_idx: i32,
    speed: f32,
    deg: f32,
    on_route: bool,
    base_deg: f32,
    fire_cnt: i32,
    barrage_pattern_idx: i32,
    field_limit_x: f32,
    field_limit_y: f32,
    app_cnt: i32,
    dst_cnt: i32,
    timeout_cnt: i32,
    z: f32,
    is_boss: bool,
    vel: Vector2,
    vel_cnt: i32,
    damaged: bool,
    boss_timer: i32,
}

impl Enemy {
    pub fn new() -> Self {
        Enemy {
            active: false,
            pos: VEC_ZERO,
            ty: EnemyType::new(0),
            battery: [Battery::new(); BATTERY_MAX],
            shield: 0,
            cnt: 0,
            top_bullet: -1,
            move_bullet: -1,
            move_point: [VEC_ZERO; MOVE_POINT_MAX],
            move_point_num: 0,
            move_point_idx: 0,
            speed: 0.0,
            deg: 0.0,
            on_route: false,
            base_deg: 0.0,
            fire_cnt: 0,
            barrage_pattern_idx: 0,
            field_limit_x: 0.0,
            field_limit_y: 0.0,
            app_cnt: 0,
            dst_cnt: 0,
            timeout_cnt: 0,
            z: 0.0,
            is_boss: false,
            vel: VEC_ZERO,
            vel_cnt: 0,
            damaged: false,
            boss_timer: 0,
        }
    }

    // Reset all top-bullet handles to "none" (-1) on (re)activation, since pooled
    // Enemy structs are reused.
    fn reset_bullet_handles(&mut self) {
        self.top_bullet = -1;
        for i in 0..BATTERY_MAX {
            for j in 0..WING_BATTERY_MAX {
                self.battery[i].top_bullet[j] = -1;
            }
        }
    }

    fn set(&mut self, px: f32, py: f32, d: f32, ty: EnemyType, move_parser: *mut c_void) {
        self.pos = Vector2 { x: px, y: py };
        self.ty = ty;
        self.reset_bullet_handles();

        self.move_bullet = bullets_add(move_parser, self.pos.x, self.pos.y, d, 0.0, 0.5, 1.0, 0, 0, 1.0, 1.0);
        if self.move_bullet < 0 {
            return;
        }
        self.cnt = 0;
        self.shield = self.ty.shield;
        for i in 0..self.ty.battery_num as usize {
            self.battery[i].shield = self.ty.battery_type[i].shield;
        }
        self.fire_cnt = 0;
        self.barrage_pattern_idx = 0;
        self.base_deg = d;
        self.app_cnt = 0;
        self.dst_cnt = 0;
        self.timeout_cnt = 0;
        self.z = 0.0;
        self.is_boss = false;
        self.active = true;
    }

    fn set_boss(&mut self, px: f32, py: f32, d: f32, ty: EnemyType) {
        self.pos = Vector2 { x: px, y: py };
        self.ty = ty;
        self.move_bullet = -1;
        self.reset_bullet_handles();

        let (hw, hh) = field_half();
        // Set the moving patterns.
        let wx = rand_next_float(hw / 4.0) + hw / 4.0;
        let wy = rand_next_float(hh / 9.0) + hh / 7.0;
        let cy = hh / 7.0 * 4.0;
        self.move_point_num = rand_next_int(3) + 2;
        for i in 0..(self.move_point_num / 2) as usize {
            self.move_point[i * 2].x = rand_next_float(wx / 2.0) + wx / 2.0;
            self.move_point[i * 2 + 1].x = -self.move_point[i * 2].x;
            let y = rand_next_signed_float(wy) + cy;
            self.move_point[i * 2].y = y;
            self.move_point[i * 2 + 1].y = y;
        }
        if self.move_point_num == 3 {
            self.move_point[2].x = 0.0;
            self.move_point[2].y = rand_next_signed_float(wy) + cy;
        }
        for _ in 0..8 {
            let idx1 = rand_next_int(self.move_point_num) as usize;
            let mut idx2 = rand_next_int(self.move_point_num) as usize;
            if idx1 == idx2 {
                idx2 += 1;
                if idx2 >= self.move_point_num as usize {
                    idx2 = 0;
                }
            }
            self.move_point.swap(idx1, idx2);
        }
        self.speed = 0.03 + rand_next_float(0.02);
        self.move_point_idx = 0;
        self.deg = PI;
        self.on_route = false;

        self.cnt = 0;
        self.shield = self.ty.shield;
        for i in 0..self.ty.battery_num as usize {
            self.battery[i].shield = self.ty.battery_type[i].shield;
        }
        for i in self.ty.battery_num as usize..BATTERY_MAX {
            self.battery[i].shield = 0;
        }
        self.fire_cnt = 0;
        self.barrage_pattern_idx = 0;
        self.base_deg = d;
        self.app_cnt = APPEARANCE_CNT;
        self.z = APPEARANCE_Z;
        self.dst_cnt = 0;
        self.timeout_cnt = 0;
        self.is_boss = true;
        self.boss_timer = 0;
        self.field_limit_x = hw / 4.0 * 3.0;
        self.field_limit_y = hh / 4.0 * 3.0;
        self.active = true;
    }

    // Fire one top-level barrage bullet at pos + ofs. Returns the bullet handle,
    // or -1 if the barrage is inactive / the pool is full.
    fn set_bullet(&self, br: &Barrage, ofs: Option<Vector2>, xr: f32) -> i32 {
        if br.rank <= 0.0 {
            return -1;
        }
        let mut bx = self.pos.x;
        let mut by = self.pos.y;
        if let Some(o) = ofs {
            bx += o.x;
            by += o.y;
        }
        if br.morph_cnt > 0 {
            bullets_add_top_morph(
                br.parser,
                bx,
                by,
                self.base_deg,
                0.0,
                br.rank,
                br.speed_rank,
                br.shape,
                br.color,
                br.bullet_size,
                br.x_reverse * xr,
                br.morph_parser.as_ptr(),
                br.morph_num,
                br.morph_cnt,
            )
        } else {
            bullets_add_top(
                br.parser,
                bx,
                by,
                self.base_deg,
                0.0,
                br.rank,
                br.speed_rank,
                br.shape,
                br.color,
                br.bullet_size,
                br.x_reverse * xr,
            )
        }
    }

    fn set_top_bullets(&mut self) {
        let bpi = self.barrage_pattern_idx as usize;
        self.top_bullet = self.set_bullet(&self.ty.barrage[bpi], None, 1.0);
        for i in 0..self.ty.battery_num as usize {
            if self.battery[i].shield <= 0 {
                continue;
            }
            let battery_num = self.ty.battery_type[i].battery_num;
            let x_reverse_alternate = self.ty.battery_type[i].x_reverse_alternate;
            let mut xr = 1.0;
            for j in 0..battery_num as usize {
                let br = self.ty.battery_type[i].barrage[bpi];
                let ofs = self.ty.battery_type[i].battery_pos[j];
                self.battery[i].top_bullet[j] = self.set_bullet(&br, Some(ofs), xr);
                if x_reverse_alternate {
                    xr = -xr;
                }
            }
        }
    }

    fn add_bonuses(&self, ofs: Option<Vector2>, sl: i32) {
        // The float subexpression matches D (computed in float, then + 0.9 in double).
        let bn = ((sl as f32 * 3.0 / ((self.cnt as f32 / 30.0) + 1.0) * bonus_get_rate()) as f64
            + 0.9) as i32;
        let (ox, oy) = match ofs {
            Some(o) => (o.x, o.y),
            None => (0.0, 0.0),
        };
        for _ in 0..bn {
            bonuses_add(self.pos.x, self.pos.y, ox, oy);
        }
    }

    fn add_fragments(&self, n: i32, z: f32, speed: f32, deg: f32) {
        let mut ni = 1;
        for i in 0..BODY_SHAPE_POINT_NUM {
            if ni >= BODY_SHAPE_POINT_NUM {
                ni = 0;
            }
            add_fragments_edge(
                n,
                self.pos.x + self.ty.body_shape_pos[i].x,
                self.pos.y + self.ty.body_shape_pos[i].y,
                self.pos.x + self.ty.body_shape_pos[ni].x,
                self.pos.y + self.ty.body_shape_pos[ni].y,
                z,
                speed,
                deg,
            );
            ni += 1;
        }
        for i in 0..self.ty.battery_num as usize {
            if self.battery[i].shield > 0 {
                add_wing_fragments(&self.ty.battery_type[i], self.pos.x, self.pos.y, n, z, speed, deg);
            }
        }
    }

    fn add_damage(&mut self, dmg: i32, slot: i32) {
        self.shield -= dmg;
        if self.shield <= 0 {
            // Destroyed.
            self.add_bonuses(None, self.ty.shield);
            score_state().increase_score(ENEMY_TYPE_SCORE[self.ty.kind as usize]);
            if self.is_boss {
                self.add_fragments(15, 0.0, 0.1, rand_next_signed_float(1.0));
                sound_manager_play_se(SE_BOSS_DESTROYED);
                screen_shake_set(20, 0.05);
                bullets_to_retro_all();
                self.remove_top_bullets();
                self.dst_cnt = DESTROYED_CNT;
            } else {
                let d;
                if self.ty.kind == TYPE_SMALL {
                    d = bullets_get_deg(self.move_bullet);
                    sound_manager_play_se(SE_ENEMY_DESTROYED);
                } else {
                    d = rand_next_signed_float(1.0);
                    sound_manager_play_se(SE_LARGE_ENEMY_DESTROYED);
                }
                self.add_fragments(self.ty.kind * 4 + 2, 0.0, 0.04, d);
                self.remove();
            }
        }
        self.damaged = true;
        let _ = slot;
    }

    fn remove_battery(&mut self, idx: usize) {
        let battery_num = self.ty.battery_type[idx].battery_num;
        for i in 0..battery_num as usize {
            if self.battery[idx].top_bullet[i] >= 0 {
                bullets_remove(self.battery[idx].top_bullet[i]);
                self.battery[idx].top_bullet[i] = -1;
            }
        }
        self.battery[idx].damaged = true;
    }

    fn add_damage_battery(&mut self, idx: usize, dmg: i32) {
        self.battery[idx].shield -= dmg;
        if self.battery[idx].shield <= 0 {
            // Wing is destroyed.
            let p = self.ty.battery_type[idx].collision_pos;
            self.add_bonuses(Some(p), self.battery[idx].shield);
            score_state().increase_score(ENEMY_WING_SCORE);
            add_wing_fragments(
                &self.ty.battery_type[idx],
                self.pos.x,
                self.pos.y,
                10,
                0.0,
                0.1,
                rand_next_signed_float(1.0),
            );
            sound_manager_play_se(SE_LARGE_ENEMY_DESTROYED);
            screen_shake_set(10, 0.03);
            self.remove_battery(idx);
            self.vel.x = -p.x / 10.0;
            self.vel.y = -p.y / 10.0;
            self.vel_cnt = 60;
            self.remove_top_bullets();
            self.fire_cnt = self.vel_cnt + 10;
        }
    }

    // Check shots and rolls hit the enemy. Returns HIT, a battery index, or NOHIT.
    fn check_hit(&self, px: f32, py: f32, xofs: f32, yofs: f32) -> i32 {
        if (px - self.pos.x).abs() < self.ty.collision_size.x + xofs
            && (py - self.pos.y).abs() < self.ty.collision_size.y + yofs
        {
            return HIT;
        }
        if self.ty.wing_collision {
            for i in 0..self.ty.battery_num as usize {
                if self.battery[i].shield <= 0 {
                    continue;
                }
                let bt = &self.ty.battery_type[i];
                if (px - self.pos.x - bt.collision_pos.x).abs() < bt.collision_size.x + xofs
                    && (py - self.pos.y - bt.collision_pos.y).abs() < bt.collision_size.y + yofs
                {
                    return i as i32;
                }
            }
        }
        NOHIT
    }

    // Check ship locks the enemy. lock_idx is the Rust lock pool slot.
    fn check_locked(&self, px: f32, py: f32, xofs: f32, lock_idx: i32) -> i32 {
        let mut lock_min_y = lock_get_lock_min_y(lock_idx);
        if (px - self.pos.x).abs() < self.ty.collision_size.x + xofs
            && self.pos.y < lock_min_y
            && self.pos.y > py
        {
            lock_set_lock_min_y(lock_idx, self.pos.y);
            return HIT;
        }
        if self.ty.wing_collision {
            let mut lp = NOHIT;
            for i in 0..self.ty.battery_num as usize {
                if self.battery[i].shield <= 0 {
                    continue;
                }
                let bt = &self.ty.battery_type[i];
                let by = self.pos.y + bt.collision_pos.y;
                if (px - self.pos.x - bt.collision_pos.x).abs() < bt.collision_size.x + xofs
                    && by < lock_min_y
                    && by > py
                {
                    lock_min_y = by;
                    lp = i as i32;
                }
            }
            if lp != NOHIT {
                lock_set_lock_min_y(lock_idx, lock_min_y);
                return lp;
            }
        }
        NOHIT
    }

    // Snapshot pushed to the Rust lock each frame: target position (battery part
    // offset folded in) and whether the lock is lost. Mirrors Lock.d's lockedPos
    // / isLockLost().
    fn compute_lock_snapshot(&self, part: i32) -> (f32, f32, bool) {
        if part < 0 {
            return (self.pos.x, self.pos.y, !self.active || self.shield <= 0);
        }
        let bt = &self.ty.battery_type[part as usize];
        (
            self.pos.x + bt.collision_pos.x,
            self.pos.y + bt.collision_pos.y,
            !self.active || self.shield <= 0 || self.battery[part as usize].shield <= 0,
        )
    }

    fn check_damage(&mut self, slot: i32) {
        // Check shots.
        for i in 0..32 {
            if !shots_is_active(i) {
                continue;
            }
            let spx = shots_get_pos_x(i);
            let spy = shots_get_pos_y(i);
            let ch = self.check_hit(spx, spy, 0.7, 0.0);
            if ch >= HIT {
                particles_init_new(spx, spy, rand_next_signed_float(0.3), 0.0, SHOT_SPEED / 4.0);
                particles_init_new(spx, spy, rand_next_signed_float(0.3), 0.0, SHOT_SPEED / 4.0);
                particles_init_new(spx, spy, PI + rand_next_signed_float(0.3), 0.0, SHOT_SPEED / 7.0);
                shots_set_inactive(i);
                if ch == HIT {
                    self.add_damage(SHOT_DAMAGE, slot);
                } else {
                    self.add_damage_battery(ch as usize, SHOT_DAMAGE);
                }
            }
        }
        if current_mode() == MODE_ROLL {
            // Check rolls.
            for i in 0..rolls_pool_size() {
                if !rolls_is_active(i) {
                    continue;
                }
                let rpx = rolls_get_pos0_x(i);
                let rpy = rolls_get_pos0_y(i);
                let ch = self.check_hit(rpx, rpy, 1.0, 1.0);
                if ch >= HIT {
                    for _ in 0..4 {
                        particles_init_new(rpx, rpy, rand_next_float(PI * 2.0), 0.0, SHOT_SPEED / 10.0);
                    }
                    let mut rd = ROLL_DAMAGE as f32;
                    if rolls_is_released(i) {
                        rd += rd;
                    } else if rolls_get_cnt(i) < ROLL_NO_COLLISION_CNT {
                        continue;
                    }
                    if ch == HIT {
                        self.add_damage(rd as i32, slot);
                    } else {
                        self.add_damage_battery(ch as usize, rd as i32);
                    }
                }
            }
        } else if self.ty.kind != TYPE_SMALL {
            // Check locks.
            for i in 0..locks_pool_size() {
                if !locks_is_active(i) {
                    continue;
                }
                let lk_state = lock_get_state(i);
                let hx = lock_get_laser_head_x(i);
                let hy = lock_get_laser_head_y(i);
                if lk_state == LOCK_SEARCH || lk_state == LOCK_SEARCHED {
                    let ch = self.check_locked(hx, hy, 2.5, i);
                    if ch >= HIT {
                        lock_set_state(i, LOCK_SEARCHED);
                        set_lock_target(i as usize, slot, ch);
                    }
                    return;
                } else if lk_state == LOCK_FIRED && lock_target_slot(i as usize) == slot {
                    let ch = self.check_hit(hx, hy, 1.5, 1.5);
                    if ch >= HIT && ch == lock_target_part(i as usize) {
                        for _ in 0..4 {
                            particles_init_new(hx, hy, rand_next_float(PI * 2.0), 0.0, SHOT_SPEED / 10.0);
                        }
                        if ch == HIT {
                            self.add_damage(LOCK_DAMAGE, slot);
                        } else {
                            self.add_damage_battery(ch as usize, LOCK_DAMAGE);
                        }
                        lock_hit(i);
                    }
                }
            }
        }
    }

    fn remove_top_bullets(&mut self) {
        if self.top_bullet >= 0 {
            bullets_remove(self.top_bullet);
            self.top_bullet = -1;
        }
        for i in 0..self.ty.battery_num as usize {
            let battery_num = self.ty.battery_type[i].battery_num;
            for j in 0..battery_num as usize {
                if self.battery[i].top_bullet[j] >= 0 {
                    bullets_remove(self.battery[i].top_bullet[j]);
                    self.battery[i].top_bullet[j] = -1;
                }
            }
        }
    }

    fn remove(&mut self) {
        self.remove_top_bullets();
        if self.move_bullet >= 0 {
            bullets_remove(self.move_bullet);
        }
        self.active = false;
    }

    fn goto_next_point(&mut self) {
        self.on_route = false;
        self.move_point_idx += 1;
        if self.move_point_idx >= self.move_point_num {
            self.move_point_idx = 0;
        }
    }

    fn move_boss(&mut self) {
        let aim = self.move_point[self.move_point_idx as usize];
        let d = (aim.x - self.pos.x).atan2(aim.y - self.pos.y);
        let mut od = d - self.deg;
        if od > PI {
            od -= PI * 2.0;
        } else if od < -PI {
            od += PI * 2.0;
        }
        let aod = od.abs();
        if aod < BOSS_MOVE_DEG {
            self.deg = d;
        } else if od > 0.0 {
            self.deg += BOSS_MOVE_DEG;
            if self.deg >= PI * 2.0 {
                self.deg -= PI * 2.0;
            }
        } else {
            self.deg -= BOSS_MOVE_DEG;
            if self.deg < 0.0 {
                self.deg += PI * 2.0;
            }
        }
        self.pos.x += self.deg.sin() * self.speed;
        self.pos.y += self.deg.cos() * self.speed;
        if self.vel_cnt > 0 {
            self.vel_cnt -= 1;
            self.pos.x += self.vel.x;
            self.pos.y += self.vel.y;
            self.vel.x *= 0.92;
            self.vel.y *= 0.92;
        }
        if !self.on_route {
            if aod < PI / 2.0 {
                self.on_route = true;
            }
        } else if aod > PI / 2.0 {
            self.goto_next_point();
        }
        if self.pos.x > self.field_limit_x {
            self.pos.x = self.field_limit_x;
            self.goto_next_point();
        } else if self.pos.x < -self.field_limit_x {
            self.pos.x = -self.field_limit_x;
            self.goto_next_point();
        }
        if self.pos.y > self.field_limit_y {
            self.pos.y = self.field_limit_y;
            self.goto_next_point();
        } else if self.pos.y < self.field_limit_y / 4.0 {
            self.pos.y = self.field_limit_y / 4.0;
            self.goto_next_point();
        }
    }

    fn control_fire_cnt(&mut self) {
        if self.fire_cnt <= 0 {
            self.set_top_bullets();
            self.fire_cnt = self.ty.fire_interval;
            self.barrage_pattern_idx += 1;
            if self.barrage_pattern_idx >= self.ty.barrage_pattern_num {
                self.barrage_pattern_idx = 0;
            }
        } else if self.fire_cnt < self.ty.fire_interval - self.ty.fire_period {
            self.remove_top_bullets();
        }
        self.fire_cnt -= 1;
    }

    fn do_move(&mut self, slot: i32) {
        enemy_type_tracker::mark(self.ty.id);
        if !self.is_boss {
            self.pos.x = bullets_get_pos_x(self.move_bullet);
            self.pos.y = bullets_get_pos_y(self.move_bullet);
        } else {
            self.move_boss();
        }
        if self.top_bullet >= 0 {
            bullets_set_pos(self.top_bullet, self.pos.x, self.pos.y);
        }
        self.damaged = false;
        for i in 0..self.ty.battery_num as usize {
            self.battery[i].damaged = false;
            let battery_num = self.ty.battery_type[i].battery_num;
            for j in 0..battery_num as usize {
                if self.battery[i].top_bullet[j] >= 0 {
                    let bp = self.ty.battery_type[i].battery_pos[j];
                    bullets_set_pos(self.battery[i].top_bullet[j], self.pos.x + bp.x, self.pos.y + bp.y);
                }
            }
        }
        if !self.is_boss {
            if field_check_hit(self.pos.x, self.pos.y) {
                self.remove();
                return;
            }
            let (_, hh) = field_half();
            if self.pos.y < -hh / 4.0 {
                self.remove_top_bullets();
            } else {
                self.control_fire_cnt();
            }
        } else {
            let mtr;
            if self.app_cnt > 0 {
                if self.z < 0.0 {
                    self.z -= APPEARANCE_Z / 60.0;
                }
                self.app_cnt -= 1;
                mtr = 1.0 - self.app_cnt as f32 / APPEARANCE_CNT as f32;
            } else if self.dst_cnt > 0 {
                self.add_fragments(1, self.z, 0.05, rand_next_signed_float(PI));
                bullets_to_retro_all();
                self.z += DESTROYED_Z / 60.0;
                self.dst_cnt -= 1;
                if self.dst_cnt <= 0 {
                    self.add_fragments(25, self.z, 0.4, rand_next_signed_float(PI));
                    sound_manager_play_se(SE_BOSS_DESTROYED);
                    screen_shake_set(60, 0.01);
                    self.remove();
                    game_manager::set_boss_shield_meter(0, 0, 0, 0, 0, 0.0);
                    return;
                }
                mtr = self.dst_cnt as f32 / DESTROYED_CNT as f32;
            } else if self.timeout_cnt > 0 {
                self.z += DESTROYED_Z / 60.0;
                self.timeout_cnt -= 1;
                if self.timeout_cnt <= 0 {
                    self.remove();
                    return;
                }
                mtr = 0.0;
            } else {
                self.control_fire_cnt();
                self.boss_timer += 1;
                if self.boss_timer > BOSS_TIMEOUT {
                    self.timeout_cnt = TIMEOUT_CNT;
                    self.shield = 0;
                    self.remove_top_bullets();
                }
                mtr = 1.0;
            }
            game_manager::set_boss_shield_meter(
                self.shield,
                self.battery[0].shield,
                self.battery[1].shield,
                self.battery[2].shield,
                self.battery[3].shield,
                mtr,
            );
        }
        self.cnt += 1;
        if self.app_cnt <= 0 && self.dst_cnt <= 0 && self.timeout_cnt <= 0 {
            self.check_damage(slot);
        }
    }

    fn draw_shape(&self) {
        let mut battery_shield = [0i32; BATTERY_MAX];
        let mut battery_damaged = [false; BATTERY_MAX];
        for i in 0..BATTERY_MAX {
            battery_shield[i] = self.battery[i].shield;
            battery_damaged[i] = self.battery[i].damaged;
        }
        draw_enemy(
            &self.ty,
            self.pos.x,
            self.pos.y,
            self.z,
            self.app_cnt,
            self.dst_cnt,
            self.timeout_cnt,
            self.damaged,
            &battery_shield,
            &battery_damaged,
        );
    }
}

impl Actor for Enemy {
    // The pool drives movement through do_move (needs the slot index); update is
    // unused for enemies.
    fn update(&mut self) {}

    fn draw(&self) {
        self.draw_shape();
    }

    fn draw_luminous(&self) {}

    fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

// Port of Effects.addFragments (Effects.d): spawns `n` fragments along the
// edge (x1,y1)-(x2,y2).
#[allow(clippy::too_many_arguments)]
fn add_fragments_edge(
    n: c_int,
    x1: c_float,
    y1: c_float,
    x2: c_float,
    y2: c_float,
    z: c_float,
    speed: c_float,
    deg: c_float,
) {
    for _ in 0..n {
        fragments_init_new(x1, y1, x2, y2, z, speed, deg);
    }
}

// Port of Enemy.addWingFragments (Enemy.d): spawns `n` fragments along each
// edge of the wing outline, offset by the enemy position (px, py).
fn add_wing_fragments(
    bt: &BatteryType,
    px: c_float,
    py: c_float,
    n: c_int,
    z: c_float,
    speed: c_float,
    deg: c_float,
) {
    let mut ni = 1;
    for i in 0..WING_SHAPE_POINT_NUM {
        if ni >= WING_SHAPE_POINT_NUM {
            ni = 0;
        }
        add_fragments_edge(
            n,
            px + bt.wing_shape_pos[i].x,
            py + bt.wing_shape_pos[i].y,
            px + bt.wing_shape_pos[ni].x,
            py + bt.wing_shape_pos[ni].y,
            z,
            speed,
            deg,
        );
        ni += 1;
    }
}

/// Port of Enemy.draw (Enemy.d). Per-frame state passed in; battery_shield /
/// battery_damaged are BATTERY_MAX-element slices.
#[allow(clippy::too_many_arguments)]
fn draw_enemy(
    et: &EnemyType,
    x: c_float,
    y: c_float,
    z: c_float,
    app_cnt: c_int,
    dst_cnt: c_int,
    timeout_cnt: c_int,
    damaged: bool,
    battery_shield: &[i32],
    battery_damaged: &[bool],
) {
    let mut ap: c_float = 0.0;
    let mut retro_z: c_float = 0.0;
    let retro_size: c_float;
    let enemy_retro_color: Color;

    if app_cnt > 0 {
        // Appearance effect of the boss.
        retro_z = z;
        ap = app_cnt as c_float / APPEARANCE_CNT as c_float;
        retro_size = et.retro_size * (1.0 + ap * 10.0);
        enemy_retro_color = Color {
            a: 1.0 - ap,
            ..et.enemy_color
        };
    } else if dst_cnt > 0 {
        retro_size = et.retro_size;
        retro_z = z;
        // f64 intermediate matches D, where `/ 2 + 0.5` evaluates in double.
        ap = ((dst_cnt as c_float / DESTROYED_CNT as c_float) as f64 / 2.0 + 0.5) as c_float;
        enemy_retro_color = Color {
            a: ap,
            ..et.enemy_color
        };
    } else if timeout_cnt > 0 {
        retro_size = et.retro_size;
        retro_z = z;
        ap = timeout_cnt as c_float / TIMEOUT_CNT as c_float;
        enemy_retro_color = Color {
            a: ap,
            ..et.enemy_color
        };
    } else {
        retro_size = et.retro_size;
        if !damaged {
            enemy_retro_color = Color {
                a: 1.0,
                ..et.enemy_color
            };
        } else {
            enemy_retro_color = Color {
                r: 1.0,
                g: 1.0,
                b: et.enemy_color.b,
                a: 1.0,
            };
        }
    }

    let mut ni = 1;
    for i in 0..BODY_SHAPE_POINT_NUM {
        if ni >= BODY_SHAPE_POINT_NUM {
            ni = 0;
        }
        draw_line_retro_with_z(
            x + et.body_shape_pos[i].x,
            y + et.body_shape_pos[i].y,
            x + et.body_shape_pos[ni].x,
            y + et.body_shape_pos[ni].y,
            retro_z,
            RETRO,
            retro_size,
            enemy_retro_color,
        );
        ni += 1;
    }

    if et.kind != TYPE_SMALL {
        unsafe { glBegin(GL_TRIANGLE_FAN) };
        set_color(Color {
            a: 0.0,
            ..enemy_retro_color
        });
        for i in 0..BODY_SHAPE_POINT_NUM {
            if i == 2 {
                set_color(enemy_retro_color);
            }
            unsafe { glVertex3f(x + et.body_shape_pos[i].x, y + et.body_shape_pos[i].y, z) };
        }
        unsafe { glEnd() };
    }

    for i in 0..et.battery_num as usize {
        let bt: &BatteryType = &et.battery_type[i];

        let battery_retro_color: Color;
        if app_cnt > 0 {
            battery_retro_color = Color {
                r: bt.r,
                g: bt.g,
                b: bt.b,
                a: 1.0 - ap,
            };
        } else if dst_cnt > 0 || timeout_cnt > 0 {
            battery_retro_color = Color {
                r: bt.r,
                g: bt.g,
                b: bt.b,
                a: ap,
            };
        } else if !battery_damaged[i] {
            battery_retro_color = Color {
                r: bt.r,
                g: bt.g,
                b: bt.b,
                a: 1.0,
            };
        } else {
            battery_retro_color = Color {
                r: 1.0,
                g: 1.0,
                b: bt.b,
                a: 1.0,
            };
        }

        ni = 1;
        if battery_shield[i] <= 0 {
            // Wing is destroyed.
            draw_line_retro_with_z(
                x + bt.wing_shape_pos[0].x,
                y + bt.wing_shape_pos[0].y,
                x + bt.wing_shape_pos[1].x,
                y + bt.wing_shape_pos[1].y,
                retro_z,
                RETRO,
                retro_size,
                battery_retro_color,
            );
        } else {
            for wi in 0..WING_SHAPE_POINT_NUM {
                if ni >= WING_SHAPE_POINT_NUM {
                    ni = 0;
                }
                draw_line_retro_with_z(
                    x + bt.wing_shape_pos[wi].x,
                    y + bt.wing_shape_pos[wi].y,
                    x + bt.wing_shape_pos[ni].x,
                    y + bt.wing_shape_pos[ni].y,
                    retro_z,
                    RETRO,
                    retro_size,
                    battery_retro_color,
                );
                ni += 1;
            }
            if et.kind != TYPE_SMALL {
                unsafe { glBegin(GL_TRIANGLE_FAN) };
                set_color(battery_retro_color);
                for wi in 0..WING_SHAPE_POINT_NUM {
                    if wi == 2 {
                        set_color(Color {
                            a: 0.0,
                            ..battery_retro_color
                        });
                    }
                    unsafe {
                        glVertex3f(x + bt.wing_shape_pos[wi].x, y + bt.wing_shape_pos[wi].y, z)
                    };
                }
                unsafe { glEnd() };
            }
        }
    }
}

// ---- Pool + lock-target association + public API ----

static mut CURRENT_MODE: c_int = 0;

fn current_mode() -> c_int {
    unsafe { CURRENT_MODE }
}

// Lock->enemy target association (= P47GameManager's lockedEnemy/lockedPart).
// The enemy is identified by its pool slot; -1 = no target.
static mut LOCK_TARGET: [(i32, i32); LOCK_NUM] = [(-1, 0); LOCK_NUM];

fn set_lock_target(i: usize, slot: i32, part: i32) {
    unsafe {
        LOCK_TARGET[i] = (slot, part);
    }
}

fn lock_target_slot(i: usize) -> i32 {
    unsafe { LOCK_TARGET[i].0 }
}

fn lock_target_part(i: usize) -> i32 {
    unsafe { LOCK_TARGET[i].1 }
}

static mut ENEMY_POOL: Option<ActorPool<Enemy>> = None;

fn enemy_pool() -> &'static mut ActorPool<Enemy> {
    unsafe { ENEMY_POOL.get_or_insert_with(|| ActorPool::new(ENEMY_MAX, Enemy::new)) }
}

pub fn enemies_clear() {
    enemy_pool().clear();
    unsafe {
        LOCK_TARGET = [(-1, 0); LOCK_NUM];
    }
}

pub fn enemies_add(
    px: f32,
    py: f32,
    d: f32,
    ty: *const EnemyType,
    move_type: c_int,
    move_type_random: c_int,
) {
    let pool = enemy_pool();
    let idx = match pool.get_instance_index() {
        Some(i) => i,
        None => return,
    };
    let ty = unsafe { *ty };
    let move_parser = barrage_get_move_parser(move_type, move_type_random);
    pool.actors[idx as usize].set(px, py, d, ty, move_parser);
}

pub fn enemies_add_boss(px: f32, py: f32, d: f32, ty: *const EnemyType) {
    let pool = enemy_pool();
    let idx = match pool.get_instance_index() {
        Some(i) => i,
        None => return,
    };
    let ty = unsafe { *ty };
    pool.actors[idx as usize].set_boss(px, py, d, ty);
}

pub fn enemies_move(mode: c_int) {
    unsafe {
        CURRENT_MODE = mode;
    }
    let pool = enemy_pool();
    let len = pool.actors.len();
    for i in 0..len {
        if pool.actors[i].active {
            pool.actors[i].do_move(i as i32);
        }
    }
}

pub fn enemies_draw() {
    enemy_pool().draw();
}

// Push each active lock's current target snapshot into the Rust lock before
// locks_update(). Replaces P47GameManager.pushLockTargets; enemies_move (which
// sets the lock->enemy association) has already run this frame.
pub fn enemies_push_lock_targets() {
    let pool = enemy_pool();
    for i in 0..LOCK_NUM {
        if !locks_is_active(i as i32) {
            continue;
        }
        let slot = lock_target_slot(i);
        if slot < 0 {
            continue;
        }
        let (px, py, lost) = pool.actors[slot as usize].compute_lock_snapshot(lock_target_part(i));
        lock_set_target_snapshot(i as i32, px, py, lost);
    }
}
