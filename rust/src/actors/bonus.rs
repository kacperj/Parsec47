use crate::actors::actor::Actor;
use crate::actors::actor_pool::ActorPool;
use crate::collision::CollisionBox;
use crate::core::rand::rand_next_signed_float;
use crate::field::field_get_collision_box;
use crate::renderer::{draw_box_line, draw_box_retro, set_color};
use crate::rendering::color::Color;
use crate::rendering::gl::{glBlendFunc, GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA};
use crate::ship::{ship_get_pos_x, ship_get_pos_y, ship_get_cnt};
use crate::state::state_export::{bonus_collected, bonus_state_reset};

const BASE_SPEED: f32 = 0.1;
const INHALE_WIDTH: f32 = 3.0;
const ACQUIRE_WIDTH: f32 = 1.0;
const RETRO_CNT: i32 = 20;
const BOX_SIZE: f32 = 0.4;
const INVINCIBLE_CNT: i32 = 228; // matches Ship.INVINCIBLE_CNT

const BONUS_COLOR: Color = Color {
    r: 0.2,
    g: 0.7,
    b: 0.5,
    a: 1.0,
};

fn bonus_draw(pos_x: f32, pos_y: f32, cnt: i32, is_down: bool, is_inhaled: bool) {
    let retro = if cnt < RETRO_CNT {
        1.0 - cnt as f32 / RETRO_CNT as f32
    } else {
        0.0
    };
    let d = cnt as f32 * 0.1;
    let ox = d.sin() * 0.3;
    let oy = d.cos() * 0.3;

    if retro > 0.0 {
        draw_box_retro(pos_x - ox, pos_y - oy, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
        draw_box_retro(pos_x + ox, pos_y + oy, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
        draw_box_retro(pos_x - oy, pos_y + ox, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
        draw_box_retro(pos_x + oy, pos_y - ox, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
    } else {
        let color = if is_inhaled {
            Color { r: 0.8, g: 0.6, b: 0.4, a: 0.7 }
        } else if is_down {
            Color { r: 0.4, g: 0.9, b: 0.6, a: 0.7 }
        } else {
            Color { r: 0.8, g: 0.9, b: 0.5, a: 0.7 }
        };
        set_color(color);
        draw_box_line(pos_x - ox - BOX_SIZE / 2.0, pos_y - oy - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
        draw_box_line(pos_x + ox - BOX_SIZE / 2.0, pos_y + oy - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
        draw_box_line(pos_x - oy - BOX_SIZE / 2.0, pos_y + ox - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
        draw_box_line(pos_x + oy - BOX_SIZE / 2.0, pos_y - ox - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
    }
}

static mut BONUS_SPEED: f32 = BASE_SPEED;
static mut BONUS_RATE: f32 = 1.0;
static mut BONUS_POOL: Option<ActorPool<BonusActor>> = None;

fn get_bonus_pool() -> &'static mut ActorPool<BonusActor> {
    unsafe { BONUS_POOL.get_or_insert_with(|| ActorPool::new(128, BonusActor::new)) }
}

pub struct BonusActor {
    active: bool,
    pos: (f32, f32),
    vel: (f32, f32),
    cnt: i32,
    is_down: bool,
    is_inhaled: bool,
    inhale_cnt: i32,
    field_limit_x: f32,
    field_limit_y: f32,
}

impl BonusActor {
    pub fn new() -> Self {
        let cb: CollisionBox = unsafe { field_get_collision_box() };
        let half_w = (cb.x2 - cb.x1) / 2.0;
        let half_h = (cb.y2 - cb.y1) / 2.0;
        BonusActor {
            active: false,
            pos: (0.0, 0.0),
            vel: (0.0, 0.0),
            cnt: 0,
            is_down: true,
            is_inhaled: false,
            inhale_cnt: 0,
            field_limit_x: half_w / 6.0 * 5.0,
            field_limit_y: half_h / 10.0 * 9.0,
        }
    }

    pub fn init(&mut self, x: f32, y: f32, ox: f32, oy: f32) {
        self.pos = (x + ox, y + oy);
        self.vel = (
            rand_next_signed_float(0.07),
            rand_next_signed_float(0.07),
        );
        self.cnt = 0;
        self.inhale_cnt = 0;
        self.is_down = true;
        self.is_inhaled = false;
        self.active = true;
    }
}

impl Actor for BonusActor {
    fn update(&mut self) {
        let speed = unsafe { BONUS_SPEED };
        self.pos.0 += self.vel.0;
        self.pos.1 += self.vel.1;
        self.vel.0 -= self.vel.0 / 50.0;
        if self.pos.0 > self.field_limit_x {
            self.pos.0 = self.field_limit_x;
            if self.vel.0 > 0.0 {
                self.vel.0 = -self.vel.0;
            }
        } else if self.pos.0 < -self.field_limit_x {
            self.pos.0 = -self.field_limit_x;
            if self.vel.0 < 0.0 {
                self.vel.0 = -self.vel.0;
            }
        }
        if self.is_down {
            self.vel.1 += (-speed - self.vel.1) / 50.0;
            if self.pos.1 < -self.field_limit_y {
                self.is_down = false;
                self.pos.1 = -self.field_limit_y;
                self.vel.1 = speed;
            }
        } else {
            self.vel.1 += (speed - self.vel.1) / 50.0;
            if self.pos.1 > self.field_limit_y {
                bonus_state_reset();
                self.active = false;
                return;
            }
        }
        self.cnt += 1;
        if self.cnt < RETRO_CNT {
            return;
        }
        let ax = (self.pos.0 - ship_get_pos_x()).abs();
        let ay = (self.pos.1 - ship_get_pos_y()).abs();
        let d = if ax > ay { ax + ay / 2.0 } else { ay + ax / 2.0 };
        let ship_cnt = ship_get_cnt();
        if d < ACQUIRE_WIDTH * (1.0 + self.inhale_cnt as f32 * 0.2)
            && ship_cnt >= -INVINCIBLE_CNT
        {
            bonus_collected();
            self.active = false;
            return;
        }
        if self.is_inhaled {
            self.inhale_cnt += 1;
            let ip = ((INHALE_WIDTH - d) / 48.0).max(0.025);
            self.vel.0 += (ship_get_pos_x() - self.pos.0) * ip;
            self.vel.1 += (ship_get_pos_y() - self.pos.1) * ip;
            if ship_cnt < -INVINCIBLE_CNT {
                self.is_inhaled = false;
                self.inhale_cnt = 0;
            }
        } else if d < INHALE_WIDTH && ship_cnt >= -INVINCIBLE_CNT {
            self.is_inhaled = true;
        }
    }

    fn draw(&self) {
        bonus_draw(self.pos.0, self.pos.1, self.cnt, self.is_down, self.is_inhaled);
    }

    fn draw_luminous(&self) {}

    fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

#[no_mangle]
pub extern "C" fn bonuses_init() {
    get_bonus_pool().clear();
}

#[no_mangle]
pub extern "C" fn bonuses_set_speed_rate(r: f32) {
    unsafe {
        BONUS_RATE = r;
        BONUS_SPEED = BASE_SPEED * r;
    }
}

#[no_mangle]
pub extern "C" fn bonus_get_rate() -> f32 {
    unsafe { BONUS_RATE }
}

#[no_mangle]
pub extern "C" fn bonuses_clear() {
    get_bonus_pool().clear();
}

#[no_mangle]
pub extern "C" fn bonuses_move() {
    get_bonus_pool().update();
}

#[no_mangle]
pub extern "C" fn bonuses_draw() {
    unsafe { glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA) };
    get_bonus_pool().draw();
    unsafe { glBlendFunc(GL_SRC_ALPHA, GL_ONE) };
}

#[no_mangle]
pub extern "C" fn bonuses_add(x: f32, y: f32, ox: f32, oy: f32) {
    if let Some(b) = get_bonus_pool().get_instance() {
        b.init(x, y, ox, oy);
    }
}
