use crate::actors::actor::Actor;
use crate::collision::{check_hit_with_space, CollisionBox};
use crate::core::vector::Vector2;
use crate::renderer::draw_box_retro;
use crate::rendering::color::Color;

const SPEED: f32 = 1.0;
const FIELD_SPACE: f32 = 1.0;
const RETRO_CNT: i32 = 4;
const SHOT_COLOR: Color = Color {
    r: 0.8,
    g: 0.8,
    b: 0.2,
    a: 0.8,
};

pub struct Shot {
    pub active: bool,
    pub pos: Vector2,
    vel: Vector2,
    deg: f32,
    cnt: i32,
    field_box: CollisionBox,
}

impl Shot {
    pub fn new() -> Self {
        Self {
            active: false,
            pos: Vector2 { x: 0.0, y: 0.0 },
            vel: Vector2 { x: 0.0, y: 0.0 },
            deg: 0.0,
            cnt: 0,
            field_box: CollisionBox {
                x1: 0.0,
                y1: 0.0,
                x2: 0.0,
                y2: 0.0,
            },
        }
    }

    pub fn init(&mut self, px: f32, py: f32, deg: f32, field_box: CollisionBox) {
        self.pos = Vector2 { x: px, y: py };
        self.deg = deg;
        self.vel = Vector2 {
            x: deg.sin(),
            y: deg.cos(),
        } * SPEED;
        self.field_box = field_box;
        self.cnt = 0;
        self.active = true;
    }
}

impl Actor for Shot {
    fn update(&mut self) {
        self.pos = self.pos + self.vel;
        if check_hit_with_space(self.field_box, self.pos.x, self.pos.y, FIELD_SPACE) {
            self.active = false;
        }
        self.cnt += 1;
    }

    fn draw(&self) {
        let retro = if self.cnt > RETRO_CNT {
            1.0
        } else {
            self.cnt as f32 / RETRO_CNT as f32
        };
        draw_box_retro(self.pos.x, self.pos.y, 0.2, 1.0, self.deg, SHOT_COLOR, retro, 0.2);
    }

    fn draw_luminous(&self) {}

    fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}
