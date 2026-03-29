use crate::actors::actor::Actor;
use crate::core::rand::*;
use crate::core::vector::Vector2;
use crate::renderer::*;
use crate::rendering::color::*;
use crate::rendering::gl::*;

const FRAGMENT_COLOR: Color = Color {
    r: 1.0,
    g: 0.8,
    b: 0.6,
    a: 1.0,
};

pub struct Fragment {
    pub active: bool,
    pos: [Vector2; 2],
    vel: [Vector2; 2],
    impact: Vector2,
    z: f32,
    lum_alp: f32,
    retro: f32,
    cnt: i32,
}

impl Actor for Fragment {
    fn update(&mut self) {
        self.cnt -= 1;
        if self.cnt < 0 {
            self.active = false;
            return;
        }
        for i in 0..2 {
            self.pos[i] = self.pos[i] + self.vel[i] + self.impact;
            self.vel[i] = self.vel[i] * 0.98;
        }
        self.impact = self.impact * 0.95;
        self.lum_alp *= 0.98;
        self.retro *= 0.97;
    }

    fn draw(&self) {
        draw_line_retro_with_z(
            self.pos[0].x,
            self.pos[0].y,
            self.pos[1].x,
            self.pos[1].y,
            self.z,
            self.retro,
            0.2,
            FRAGMENT_COLOR,
        );
    }

    fn draw_luminous(&self) {
        if self.lum_alp < 0.2 {
            return;
        }
        set_color(Color {
            r: FRAGMENT_COLOR.r,
            g: FRAGMENT_COLOR.g,
            b: FRAGMENT_COLOR.b,
            a: self.lum_alp,
        });
        unsafe {
            glVertex3f(self.pos[0].x, self.pos[0].y, self.z);
            glVertex3f(self.pos[1].x, self.pos[1].y, self.z);
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Fragment {
    pub fn new() -> Self {
        Self {
            active: false,
            pos: [Vector2 { x: 0.0, y: 0.0 }; 2],
            vel: [Vector2 { x: 0.0, y: 0.0 }; 2],
            impact: Vector2 { x: 0.0, y: 0.0 },
            z: 0.0,
            lum_alp: 0.0,
            retro: 0.0,
            cnt: 0,
        }
    }

    pub fn init(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, z: f32, speed: f32, deg: f32) {
        let r1 = rand_next_float(1.0);
        let r2 = rand_next_float(1.0);

        self.pos[0] = Vector2 {
            x: x1 * r1 + x2 * (1.0 - r1),
            y: y1 * r1 + y2 * (1.0 - r1),
        };
        self.pos[1] = Vector2 {
            x: x1 * r2 + x2 * (1.0 - r2),
            y: y1 * r2 + y2 * (1.0 - r2),
        };

        for i in 0..2 {
            self.vel[i] = Vector2 {
                x: rand_next_signed_float(1.0),
                y: rand_next_signed_float(1.0),
            } * speed;
        }

        self.impact = Vector2 {
            x: deg.sin(),
            y: deg.cos(),
        } * (speed * 4.0);
        self.z = z;
        self.cnt = 32 + rand_next_int(24);
        self.lum_alp = 0.8 + rand_next_float(0.2);
        self.retro = 1.0;
        self.active = true;
    }
}
