use crate::actors::actor::Actor;
use crate::core::rand::*;
use crate::core::vector::Vector2;
use crate::renderer::*;
use crate::rendering::color::*;
use crate::rendering::gl::*;

const PARTICLE_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.5,
    a: 1.0,
};

pub struct Particle {
    pub active: bool,
    pos: Vector2,
    previous_pos: Vector2,
    vel: Vector2,
    life: i32,
    lum_alp: f32,
    z: f32,
    pz: f32,
    mz: f32,
}

impl Actor for Particle {
    fn update(&mut self) {
        self.life -= 1;

        if self.life < 0 {
            self.active = false;
            return;
        }

        self.previous_pos = self.pos;
        self.pz = self.z;

        self.pos = self.pos + self.vel;
        self.vel = self.vel * 0.98;
        self.z += self.mz;

        self.lum_alp *= 0.98;
    }

    fn draw(&self) {
        set_color(PARTICLE_COLOR);

        unsafe {
            glVertex3f(self.previous_pos.x, self.previous_pos.y, self.pz);
            glVertex3f(self.pos.x, self.pos.y, self.z);
        }
    }

    fn draw_luminous(&self) {
        if self.lum_alp < 0.2 {
            return;
        }
        set_color(Color {
            r: PARTICLE_COLOR.r,
            g: PARTICLE_COLOR.g,
            b: PARTICLE_COLOR.b,
            a: self.lum_alp,
        });

        unsafe {
            glVertex3f(self.previous_pos.x, self.previous_pos.y, self.pz);
            glVertex3f(self.pos.x, self.pos.y, self.z);
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Particle {
    pub fn new() -> Self {
        Self {
            pos: Vector2 { x: 0.0, y: 0.0 },
            previous_pos: Vector2 { x: 0.0, y: 0.0 },
            vel: Vector2 { x: 0.0, y: 0.0 },
            life: 0,
            active: false,
            lum_alp: 0.0,
            z: 0.0,
            pz: 0.0,
            mz: 0.0,
        }
    }

    pub fn init(&mut self, p: Vector2, d: f32, ofs: f32, speed: f32) {
        let direction_vector = Vector2 {
            x: d.sin(),
            y: d.cos(),
        };

        let offset_vector = direction_vector * ofs;

        if ofs > 0.0 {
            self.pos = p + offset_vector;
        } else {
            self.pos = p;
        }

        self.z = 0.0;
        let sb = rand_next_float(0.5) + 0.75;

        self.vel = direction_vector * speed * sb;

        self.mz = rand_next_signed_float(0.7);
        self.life = 12 + rand_next_int(48);
        self.lum_alp = 0.8 + rand_next_float(0.2);
        self.active = true;
    }
}
