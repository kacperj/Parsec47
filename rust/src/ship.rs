use crate::actors::actor_export::{
    fragments_init_new, particles_init_new, rolls_init_new, rolls_release_all, shots_init_new,
};
use crate::bullets::bullet_actor_pool::bullets_set_target;
use crate::core::rand::rand_next_float;
use crate::field::field_get_collision_box;
use crate::pad::{pad_get_button_state, pad_get_pad_state};
use crate::renderer::{draw_box_line, draw_box_solid, set_color};
use crate::rendering::color::Color;
use crate::rendering::gl::*;
use crate::sound::sound_manager_play_se;
use crate::state::state_export::score_state;
use core::f32::consts::PI;
use core::ffi::{c_float, c_int};

const BASE_SPEED: f32 = 0.6;
const SLOW_BASE_SPEED: f32 = 0.3;
const BANK_BASE: f32 = 50.0;
const FIRE_WIDE_BASE_DEG: f32 = 0.7;
const FIRE_NARROW_BASE_DEG: f32 = 0.5;
const TURRET_INTERVAL_LENGTH: f32 = 0.2;
const FIELD_SPACE: f32 = 1.5;
const RESTART_CNT: i32 = 300;
const INVINCIBLE_CNT: i32 = 228;

const MODE_ROLL: c_int = 0;

// Pad bitmask values (mirrors pad.rs).
const PAD_UP: c_int = 1;
const PAD_DOWN: c_int = 2;
const PAD_LEFT: c_int = 4;
const PAD_RIGHT: c_int = 8;
const PAD_BUTTON1: c_int = 16;
const PAD_BUTTON2: c_int = 32;

// SE indices (mirrors SoundManager.d enum).
const SE_SHOT: c_int = 0;
const SE_ROLL_CHARGE: c_int = 1;
const SE_ROLL_RELEASE: c_int = 2;
const SE_SHIP_DESTROYED: c_int = 3;

// Event bits returned to the game manager so it can react synchronously.
const SHIP_EVENT_ADD_LOCK: c_int = 1;
const SHIP_EVENT_RELEASE_LOCK: c_int = 2;
const SHIP_EVENT_DESTROYED: c_int = 4;

struct ShipState {
    ppos_x: f32,
    ppos_y: f32,
    vel_x: f32,
    vel_y: f32,
    speed: f32,
    base_speed: f32,
    slow_speed: f32,
    bank: f32,
    fire_wide_deg: f32,
    fire_cnt: i32,
    ttl_cnt: i32,
    field_limit_x: f32,
    field_limit_y: f32,
    roll_lock_cnt: i32,
    roll_charged: bool,
    mode: c_int,
    is_slow: bool,
}

static mut SHIP: ShipState = ShipState {
    ppos_x: 0.0,
    ppos_y: 0.0,
    vel_x: 0.0,
    vel_y: 0.0,
    speed: BASE_SPEED,
    base_speed: BASE_SPEED,
    slow_speed: SLOW_BASE_SPEED,
    bank: 0.0,
    fire_wide_deg: FIRE_WIDE_BASE_DEG,
    fire_cnt: 0,
    ttl_cnt: 0,
    field_limit_x: 0.0,
    field_limit_y: 0.0,
    roll_lock_cnt: 0,
    roll_charged: false,
    mode: MODE_ROLL,
    is_slow: false,
};

static mut SHIP_POS_X: c_float = 0.0;
static mut SHIP_POS_Y: c_float = 0.0;
static mut SHIP_CNT: i32 = 0;
static mut SHIP_DISPLAY_LIST_IDX: GLuint = 0;

pub fn ship_get_pos_x() -> c_float {
    unsafe { SHIP_POS_X }
}

pub fn ship_get_pos_y() -> c_float {
    unsafe { SHIP_POS_Y }
}

pub fn ship_set_cnt(cnt: i32) {
    unsafe { SHIP_CNT = cnt; }
}

pub fn ship_get_cnt() -> i32 {
    unsafe { SHIP_CNT }
}

pub fn ship_set_slow(v: c_int) {
    unsafe { SHIP.is_slow = v != 0; }
}

pub fn ship_set_speed_rate(rate: c_float) {
    unsafe {
        if !SHIP.is_slow {
            SHIP.base_speed = BASE_SPEED * rate;
        } else {
            SHIP.base_speed = BASE_SPEED * 0.7;
        }
        SHIP.slow_speed = SLOW_BASE_SPEED * rate;
    }
}

pub fn ship_start(mode: c_int) {
    let fb = field_get_collision_box();
    let half_width = (fb.x2 - fb.x1) / 2.0;
    let half_height = (fb.y2 - fb.y1) / 2.0;
    unsafe {
        SHIP.mode = mode;
        SHIP.field_limit_x = half_width - FIELD_SPACE;
        SHIP.field_limit_y = half_height - FIELD_SPACE;
        SHIP_POS_X = 0.0;
        SHIP_POS_Y = -half_height / 2.0;
        SHIP.ppos_x = SHIP_POS_X;
        SHIP.ppos_y = SHIP_POS_Y;
        SHIP.vel_x = 0.0;
        SHIP.vel_y = 0.0;
        SHIP.speed = BASE_SPEED;
        SHIP.fire_wide_deg = FIRE_WIDE_BASE_DEG;
        SHIP_CNT = -INVINCIBLE_CNT;
        SHIP.fire_cnt = 0;
        SHIP.roll_lock_cnt = 0;
        SHIP.bank = 0.0;
        SHIP.roll_charged = false;
    }
    score_state().reset_bonus_score();
}

pub fn ship_move() -> c_int {
    let mut events: c_int = 0;
    unsafe {
        SHIP_CNT += 1;
        if SHIP_CNT < -INVINCIBLE_CNT {
            return events;
        }

        let button = pad_get_button_state();
        if button & PAD_BUTTON2 != 0 {
            SHIP.speed += (SHIP.slow_speed - SHIP.speed) * 0.2;
            SHIP.fire_wide_deg += (FIRE_NARROW_BASE_DEG - SHIP.fire_wide_deg) * 0.1;
            SHIP.roll_lock_cnt += 1;
            if SHIP.mode == MODE_ROLL {
                if SHIP.roll_lock_cnt % 15 == 0 {
                    rolls_init_new();
                    sound_manager_play_se(SE_ROLL_CHARGE);
                    SHIP.roll_charged = true;
                }
            } else if SHIP.roll_lock_cnt % 10 == 0 {
                events |= SHIP_EVENT_ADD_LOCK;
            }
        } else {
            SHIP.speed += (SHIP.base_speed - SHIP.speed) * 0.2;
            SHIP.fire_wide_deg += (FIRE_WIDE_BASE_DEG - SHIP.fire_wide_deg) * 0.1;
            if SHIP.mode == MODE_ROLL {
                if SHIP.roll_charged {
                    SHIP.roll_lock_cnt = 0;
                    rolls_release_all();
                    sound_manager_play_se(SE_ROLL_RELEASE);
                    SHIP.roll_charged = false;
                }
            } else {
                SHIP.roll_lock_cnt = 0;
                events |= SHIP_EVENT_RELEASE_LOCK;
            }
        }

        let pad = pad_get_pad_state();
        SHIP.vel_x = 0.0;
        SHIP.vel_y = 0.0;
        if pad & PAD_UP != 0 {
            SHIP.vel_y = SHIP.speed;
        } else if pad & PAD_DOWN != 0 {
            SHIP.vel_y = -SHIP.speed;
        }
        if pad & PAD_RIGHT != 0 {
            SHIP.vel_x = SHIP.speed;
        } else if pad & PAD_LEFT != 0 {
            SHIP.vel_x = -SHIP.speed;
        }
        if SHIP.vel_x != 0.0 && SHIP.vel_y != 0.0 {
            SHIP.vel_x *= 0.707;
            SHIP.vel_y *= 0.707;
        }
        SHIP.ppos_x = SHIP_POS_X;
        SHIP.ppos_y = SHIP_POS_Y;
        SHIP_POS_X += SHIP.vel_x;
        SHIP_POS_Y += SHIP.vel_y;
        SHIP.bank += (SHIP.vel_x * BANK_BASE - SHIP.bank) * 0.1;
        if SHIP_POS_X < -SHIP.field_limit_x {
            SHIP_POS_X = -SHIP.field_limit_x;
        } else if SHIP_POS_X > SHIP.field_limit_x {
            SHIP_POS_X = SHIP.field_limit_x;
        }
        if SHIP_POS_Y < -SHIP.field_limit_y {
            SHIP_POS_Y = -SHIP.field_limit_y;
        } else if SHIP_POS_Y > SHIP.field_limit_y {
            SHIP_POS_Y = SHIP.field_limit_y;
        }

        if button & PAD_BUTTON1 != 0 {
            let fire_pos_x;
            let td;
            match SHIP.fire_cnt % 4 {
                0 => {
                    fire_pos_x = SHIP_POS_X + TURRET_INTERVAL_LENGTH;
                    td = 0.0;
                }
                1 => {
                    fire_pos_x = SHIP_POS_X + TURRET_INTERVAL_LENGTH;
                    td = SHIP.fire_wide_deg * ((SHIP.fire_cnt / 4 % 5) as f32) * 0.2;
                }
                2 => {
                    fire_pos_x = SHIP_POS_X - TURRET_INTERVAL_LENGTH;
                    td = 0.0;
                }
                _ => {
                    fire_pos_x = SHIP_POS_X - TURRET_INTERVAL_LENGTH;
                    td = -SHIP.fire_wide_deg * ((SHIP.fire_cnt / 4 % 5) as f32) * 0.2;
                }
            }
            let fb = field_get_collision_box();
            shots_init_new(fire_pos_x, SHIP_POS_Y, td, fb.x1, fb.y1, fb.x2, fb.y2);
            sound_manager_play_se(SE_SHOT);
            SHIP.fire_cnt += 1;
        }

        SHIP.ttl_cnt += 1;

        // Reached only when SHIP_CNT >= -INVINCIBLE_CNT (the early return above
        // handles the rest), matching the old D guard in Ship.move().
        bullets_set_target(SHIP_POS_X, SHIP_POS_Y);
    }
    events
}

pub fn ship_destroyed() -> c_int {
    let mut events: c_int = 0;
    unsafe {
        if SHIP_CNT <= 0 {
            return events;
        }
        sound_manager_play_se(SE_SHIP_DESTROYED);
        if SHIP.mode == MODE_ROLL {
            rolls_release_all();
        } else {
            events |= SHIP_EVENT_RELEASE_LOCK;
        }
        events |= SHIP_EVENT_DESTROYED;
        let px = SHIP_POS_X;
        let py = SHIP_POS_Y;
        for _ in 0..30 {
            fragments_init_new(px, py, px, py, 0.0, 0.08, PI);
        }
        for _ in 0..45 {
            particles_init_new(px, py, rand_next_float(PI * 2.0), 0.0, 0.6);
        }
        let mode = SHIP.mode;
        ship_start(mode);
        SHIP_CNT = -RESTART_CNT;
    }
    events
}

pub fn ship_create_display_lists() {
    unsafe {
        SHIP_DISPLAY_LIST_IDX = glGenLists(3);

        // List 0: wing segment
        glNewList(SHIP_DISPLAY_LIST_IDX, GL_COMPILE);
        set_color(Color { r: 0.5, g: 1.0, b: 0.5, a: 0.2 });
        draw_box_solid(-0.1, -0.5, 0.2, 1.0);
        set_color(Color { r: 0.5, g: 1.0, b: 0.5, a: 0.4 });
        draw_box_line(-0.1, -0.5, 0.2, 1.0);
        glEndList();

        // List 1: body core
        glNewList(SHIP_DISPLAY_LIST_IDX + 1, GL_COMPILE);
        set_color(Color { r: 1.0, g: 0.2, b: 0.2, a: 1.0 });
        draw_box_solid(-0.2, -0.2, 0.4, 0.4);
        set_color(Color { r: 1.0, g: 0.5, b: 0.5, a: 1.0 });
        draw_box_line(-0.2, -0.2, 0.4, 0.4);
        glEndList();

        // List 2: engine thruster
        glNewList(SHIP_DISPLAY_LIST_IDX + 2, GL_COMPILE);
        set_color(Color { r: 0.7, g: 1.0, b: 0.5, a: 0.3 });
        draw_box_solid(-0.15, -0.3, 0.3, 0.6);
        set_color(Color { r: 0.7, g: 1.0, b: 0.5, a: 0.6 });
        draw_box_line(-0.15, -0.3, 0.3, 0.6);
        glEndList();
    }
}

pub fn ship_draw() {
    let cnt = unsafe { SHIP_CNT };
    let bank = unsafe { SHIP.bank };
    let fire_wide_deg = unsafe { SHIP.fire_wide_deg };
    let ttl_cnt = unsafe { SHIP.ttl_cnt };
    if cnt < -INVINCIBLE_CNT || (cnt < 0 && (-cnt % 32) < 16) {
        return;
    }
    let pos_x = unsafe { SHIP_POS_X };
    let pos_y = unsafe { SHIP_POS_Y };
    let dl = unsafe { SHIP_DISPLAY_LIST_IDX };
    unsafe {
        // Left wing assembly
        glPushMatrix();
        glTranslatef(pos_x, pos_y, 0.0);
        glCallList(dl + 1);
        glRotatef(bank, 0.0, 1.0, 0.0);
        glTranslatef(-0.5, 0.0, 0.0);
        glCallList(dl);
        glTranslatef(0.2, 0.3, 0.2);
        glCallList(dl);
        glTranslatef(0.0, 0.0, -0.4);
        glCallList(dl);
        glPopMatrix();
        // Right wing assembly
        glPushMatrix();
        glTranslatef(pos_x, pos_y, 0.0);
        glRotatef(bank, 0.0, 1.0, 0.0);
        glTranslatef(0.5, 0.0, 0.0);
        glCallList(dl);
        glTranslatef(-0.2, 0.3, 0.2);
        glCallList(dl);
        glTranslatef(0.0, 0.0, -0.4);
        glCallList(dl);
        glPopMatrix();
        // Engine thrusters (6 per side)
        for i in 0..6i32 {
            glPushMatrix();
            glTranslatef(pos_x - 0.7, pos_y - 0.3, 0.0);
            glRotatef(bank, 0.0, 1.0, 0.0);
            glRotatef(180.0 / 2.0 - fire_wide_deg * 100.0, 0.0, 0.0, 1.0);
            glRotatef(i as c_float * 180.0 / 3.0 - ttl_cnt as c_float * 4.0, 1.0, 0.0, 0.0);
            glTranslatef(0.0, 0.0, 0.7);
            glCallList(dl + 2);
            glPopMatrix();
            glPushMatrix();
            glTranslatef(pos_x + 0.7, pos_y - 0.3, 0.0);
            glRotatef(bank, 0.0, 1.0, 0.0);
            glRotatef(-180.0 / 2.0 + fire_wide_deg * 100.0, 0.0, 0.0, 1.0);
            glRotatef(i as c_float * 180.0 / 3.0 - ttl_cnt as c_float * 4.0, 1.0, 0.0, 0.0);
            glTranslatef(0.0, 0.0, 0.7);
            glCallList(dl + 2);
            glPopMatrix();
        }
    }
}
