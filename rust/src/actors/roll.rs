use crate::actors::actor::Actor;
use crate::core::vector::Vector2;
use crate::field::field_get_collision_box;
use crate::renderer::draw_box_retro;
use crate::rendering::color::Color;

const LENGTH: usize = 4;
const BASE_LENGTH: f32 = 1.0;
const BASE_RESISTANCE: f32 = 0.8;
const BASE_SPRING: f32 = 0.2;
const BASE_SIZE: f32 = 0.2;
const BASE_DIST: f32 = 3.0;
const SPEED: f32 = 0.75;
const ROLL_COLOR: Color = Color {
    r: 1.0,
    g: 0.8,
    b: 0.5,
    a: 1.0,
};

pub struct Roll {
    pub active: bool,
    pub released: bool,
    pub pos: [Vector2; LENGTH],
    pub cnt: i32,
    vel: [Vector2; LENGTH],
    dist: f32,
}

impl Roll {
    pub fn new() -> Self {
        let zero = Vector2 { x: 0.0, y: 0.0 };
        Self {
            active: false,
            released: false,
            pos: [zero; LENGTH],
            cnt: 0,
            vel: [zero; LENGTH],
            dist: 0.0,
        }
    }

    pub fn init(&mut self, ship_x: f32, ship_y: f32) {
        let zero = Vector2 { x: 0.0, y: 0.0 };
        for i in 0..LENGTH {
            self.pos[i] = Vector2 { x: ship_x, y: ship_y };
            self.vel[i] = zero;
        }
        self.cnt = 0;
        self.dist = 0.0;
        self.released = false;
        self.active = true;
    }

    pub fn tick(&mut self, ship_x: f32, ship_y: f32, emit_particle: impl Fn(f32, f32)) {
        if self.released {
            self.pos[0].y += SPEED;
            let half_h = unsafe { field_get_collision_box() }.y2;
            if self.pos[0].y > half_h {
                self.active = false;
                return;
            }
            emit_particle(self.pos[0].x, self.pos[0].y);
        } else {
            if self.dist < BASE_DIST {
                self.dist += BASE_DIST / 90.0;
            }
            let angle = self.cnt as f32 * 0.1;
            self.pos[0].x = ship_x + angle.sin() * self.dist;
            self.pos[0].y = ship_y + angle.cos() * self.dist;
        }
        for i in 1..LENGTH {
            self.pos[i].x += self.vel[i].x;
            self.pos[i].y += self.vel[i].y;
            self.vel[i].x *= BASE_RESISTANCE;
            self.vel[i].y *= BASE_RESISTANCE;
            let dx = self.pos[i - 1].x - self.pos[i].x;
            let dy = self.pos[i - 1].y - self.pos[i].y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > BASE_LENGTH {
                let v = (dist - BASE_LENGTH) * BASE_SPRING;
                // D calls atan2(dx, dy) — heading from +Y axis; Rust dx.atan2(dy) matches exactly
                let deg = dx.atan2(dy);
                self.vel[i].x += deg.sin() * v;
                self.vel[i].y += deg.cos() * v;
            }
        }
        self.cnt += 1;
    }
}

impl Actor for Roll {
    fn update(&mut self) {} // bypassed — rolls_update iterates and calls tick() directly
    fn draw(&self) {
        let (retro, retro_size) = if self.released {
            (1.0f32, 0.2f32)
        } else {
            (0.5f32, 0.2f32)
        };
        for i in 0..LENGTH {
            let size = BASE_SIZE * (LENGTH - i) as f32;
            draw_box_retro(
                self.pos[i].x,
                self.pos[i].y,
                size,
                size,
                self.cnt as f32 * 0.1,
                ROLL_COLOR,
                retro,
                retro_size,
            );
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
