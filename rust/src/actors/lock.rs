use crate::actors::actor::Actor;
use crate::actors::actor_export::particles_init_new;
use crate::core::rand::rand_next_signed_float;
use crate::core::vector::Vector2;
use crate::field::field_get_collision_box;
use crate::renderer::{draw_box_retro, draw_line_retro_with_z};
use crate::rendering::color::Color;
use crate::sound::sound_manager_play_se;
use core::ffi::c_int;

const LENGTH: usize = 12;
const LOCK_ANIM_DURATION_I: i32 = 8;
const LOCK_ANIM_DURATION: f32 = 8.0;
const NO_COLLISION_CNT: i32 = 8;
const SPEED: f32 = 0.01;

const LOCK_COLOR: Color = Color {
    r: 1.0,
    g: 0.8,
    b: 0.5,
    a: 1.0,
};

// Lock state machine (mirrors the enum in src/abagames/p47/Lock.d).
const STATE_SEARCH: c_int = 0;
const STATE_SEARCHED: c_int = 1;
const STATE_LOCKING: c_int = 2;
const STATE_LOCKED: c_int = 3;
const STATE_FIRED: c_int = 4;
const STATE_HIT: c_int = 5;
const STATE_CANCELED: c_int = 6;

// SE indices (mirror SoundManager.d enum).
const SE_LOCK: c_int = 9;
const SE_LASER: c_int = 10;

fn field_half_height() -> f32 {
    let b = field_get_collision_box();
    (b.y2 - b.y1) * 0.5
}

pub struct Lock {
    pub active: bool,
    pub state: c_int,
    pub lock_min_y: f32,
    pub released: bool,
    // Per-frame target snapshot pushed from the enemy. Replaces the direct
    // lockedEnemy reads in Lock.d (pos and isLockLost()). The battery-part offset
    // is already folded into locked_pos by the pusher, so no part index is needed.
    pub locked_pos: Vector2,
    pub lock_lost: bool,
    vel: Vector2,
    laser_trace: [Vector2; LENGTH],
    lock_anim_progress: i32,
}

impl Lock {
    pub fn new() -> Self {
        let zero = Vector2 { x: 0.0, y: 0.0 };
        Self {
            active: false,
            state: STATE_SEARCH,
            lock_min_y: 0.0,
            released: false,
            locked_pos: zero,
            lock_lost: false,
            vel: zero,
            laser_trace: [zero; LENGTH],
            lock_anim_progress: 0,
        }
    }

    fn reset(&mut self) {
        let head = Vector2 {
            x: crate::ship::ship_get_pos_x(),
            y: crate::ship::ship_get_pos_y(),
        };
        for i in 0..LENGTH {
            self.laser_trace[i] = head;
        }
        self.vel.x = rand_next_signed_float(1.5);
        self.vel.y = -2.0;
        self.lock_anim_progress = 0;
    }

    pub fn set(&mut self) {
        self.reset();
        self.state = STATE_SEARCH;
        self.lock_min_y = field_half_height() * 2.0;
        self.released = false;
        self.lock_lost = false;
        self.active = true;
    }

    pub fn hit(&mut self) {
        self.state = STATE_HIT;
        self.lock_anim_progress = 0;
    }

    pub fn laser_head(&self) -> Vector2 {
        self.laser_trace[0]
    }

    pub fn tick(&mut self) {
        if self.state == STATE_SEARCH {
            self.active = false;
            return;
        } else if self.state == STATE_SEARCHED {
            self.state = STATE_LOCKING;
            sound_manager_play_se(SE_LOCK);
        }
        // (lockedPos is supplied by the pushed snapshot; no enemy read here.)

        match self.state {
            STATE_LOCKING => {
                if self.lock_anim_progress >= LOCK_ANIM_DURATION_I {
                    self.state = STATE_LOCKED;
                    sound_manager_play_se(SE_LASER);
                    self.lock_anim_progress = 0;
                }
            }
            STATE_LOCKED | STATE_FIRED | STATE_CANCELED => {
                if self.state == STATE_LOCKED && self.lock_anim_progress >= NO_COLLISION_CNT {
                    self.state = STATE_FIRED;
                }
                if self.state != STATE_CANCELED {
                    let direction_to_target = self.locked_pos - self.laser_trace[0];
                    if self.lock_lost {
                        self.state = STATE_CANCELED;
                    } else {
                        self.vel = self.vel + direction_to_target * SPEED;
                    }
                    self.vel = self.vel * 0.9;
                    self.laser_trace[0] = self.laser_trace[0]
                        + direction_to_target * (0.002 * self.lock_anim_progress as f32);
                } else {
                    self.vel.y += (field_half_height() * 2.0 - self.laser_trace[0].y) * SPEED;
                }

                for i in (1..LENGTH).rev() {
                    self.laser_trace[i] = self.laser_trace[i - 1];
                }
                self.laser_trace[0] = self.laser_trace[0] + self.vel;

                if self.laser_trace[0].y > field_half_height() + 5.0 {
                    if self.state == STATE_CANCELED {
                        self.active = false;
                        return;
                    } else {
                        self.state = STATE_LOCKED;
                        sound_manager_play_se(SE_LASER);
                        self.reset();
                    }
                }
                // D calls atan2(dx, dy) — heading from +Y axis; Rust dx.atan2(dy) matches.
                let d = (self.laser_trace[1].x - self.laser_trace[0].x)
                    .atan2(self.laser_trace[1].y - self.laser_trace[0].y);
                particles_init_new(self.laser_trace[0].x, self.laser_trace[0].y, d, 0.0, SPEED * 32.0);
            }
            STATE_HIT => {
                for i in 1..LENGTH {
                    self.laser_trace[i] = self.laser_trace[i - 1];
                }
                if self.lock_anim_progress > 5 {
                    if !self.released {
                        self.state = STATE_LOCKED;
                        sound_manager_play_se(SE_LASER);
                        self.reset();
                    } else {
                        self.active = false;
                        return;
                    }
                }
            }
            _ => {}
        }
        self.lock_anim_progress += 1;
    }
}

fn draw_lock_marker(center_x: f32, center_y: f32, r: f32, mut d: f32, retro: f32, retro_size: f32) {
    for _ in 0..3 {
        draw_box_retro(
            center_x + d.sin() * r,
            center_y + d.cos() * r,
            0.2,
            1.0,
            d + 3.14 / 2.0,
            LOCK_COLOR,
            retro,
            retro_size,
        );
        d += 6.28 / 3.0;
    }
}

impl Actor for Lock {
    fn update(&mut self) {
        self.tick();
    }

    fn draw(&self) {
        match self.state {
            STATE_LOCKING => {
                let animation_progress = LOCK_ANIM_DURATION - self.lock_anim_progress as f32;
                let y = self.locked_pos.y - animation_progress * 0.5;
                let d = animation_progress * 0.1;
                let r = animation_progress * 0.5 + 0.8;
                let retro = animation_progress / LOCK_ANIM_DURATION;
                draw_lock_marker(self.locked_pos.x, y, r, d, retro, 0.2);
            }
            STATE_LOCKED | STATE_FIRED | STATE_CANCELED | STATE_HIT => {
                draw_lock_marker(self.locked_pos.x, self.locked_pos.y, 0.8, 0.0, 0.0, 0.2);

                let mut r = self.lock_anim_progress as f32 * 0.1;
                for i in 0..LENGTH - 1 {
                    let rr = r.clamp(0.0, 1.0);
                    draw_line_retro_with_z(
                        self.laser_trace[i].x,
                        self.laser_trace[i].y,
                        self.laser_trace[i + 1].x,
                        self.laser_trace[i + 1].y,
                        0.0,
                        rr,
                        0.33,
                        LOCK_COLOR,
                    );
                    r -= 0.1;
                }
            }
            _ => {}
        }
    }

    fn draw_luminous(&self) {}

    fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}
