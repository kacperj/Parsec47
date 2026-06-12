//! Port of src/abagames/p47/bullets/Bullet.d — a single BulletML-controlled bullet
//! plus p47-specific params (speedRank, shape, color, size, xReverse).
use crate::bullets::ffi::*;
use crate::core::vector::Vector2;
use std::os::raw::c_void;
use std::ptr;

pub const MORPH_MAX: usize = 8;

#[derive(Copy, Clone)]
pub struct Bullet {
    pub pos: Vector2,
    pub acc: Vector2,
    pub deg: f32,
    pub speed: f32,
    pub rank: f32,
    pub id: i32,
    pub morph_num: i32,
    pub morph_idx: i32,
    pub morph_cnt: i32,
    pub base_morph_idx: i32,
    pub base_morph_cnt: i32,
    pub is_morph: bool,
    pub morph_parser: [*mut c_void; MORPH_MAX],
    pub speed_rank: f32,
    pub shape: i32,
    pub color: i32,
    pub bullet_size: f32,
    pub x_reverse: f32,
    runner: *mut c_void,
}

impl Bullet {
    pub fn new(id: i32) -> Self {
        Bullet {
            pos: Vector2 { x: 0.0, y: 0.0 },
            acc: Vector2 { x: 0.0, y: 0.0 },
            deg: 0.0,
            speed: 0.0,
            rank: 0.0,
            id,
            morph_num: 0,
            morph_idx: 0,
            morph_cnt: 0,
            base_morph_idx: 0,
            base_morph_cnt: 0,
            is_morph: false,
            morph_parser: [ptr::null_mut(); MORPH_MAX],
            speed_rank: 0.0,
            shape: 0,
            color: 0,
            bullet_size: 0.0,
            x_reverse: 0.0,
            runner: ptr::null_mut(),
        }
    }

    pub fn set_param(&mut self, sr: f32, sh: i32, cl: i32, sz: f32, xr: f32) {
        self.speed_rank = sr;
        self.shape = sh;
        self.color = cl;
        self.bullet_size = sz;
        self.x_reverse = xr;
    }

    pub fn set_morph(&mut self, mrp: &[*mut c_void], num: i32, idx: i32, cnt: i32) {
        if cnt <= 0 {
            self.is_morph = false;
            return;
        }
        self.is_morph = true;
        self.morph_cnt = cnt;
        self.base_morph_cnt = cnt;
        self.morph_num = num;
        for i in 0..num as usize {
            self.morph_parser[i] = mrp[i];
        }
        self.morph_idx = idx;
        if self.morph_idx >= self.morph_num {
            self.morph_idx = 0;
        }
        self.base_morph_idx = self.morph_idx;
    }

    pub fn reset_morph(&mut self) {
        self.morph_idx = self.base_morph_idx;
        self.morph_cnt = self.base_morph_cnt;
    }

    pub fn set(&mut self, x: f32, y: f32, deg: f32, speed: f32, rank: f32) {
        self.pos.x = x;
        self.pos.y = y;
        self.acc.x = 0.0;
        self.acc.y = 0.0;
        self.deg = deg;
        self.speed = speed;
        self.rank = rank;
        self.runner = ptr::null_mut();
    }

    pub fn set_runner(&mut self, runner: *mut c_void) {
        self.runner = runner;
    }

    pub fn runner(&self) -> *mut c_void {
        self.runner
    }

    pub fn remove(&mut self) {
        if !self.runner.is_null() {
            unsafe { BulletMLRunner_delete(self.runner) };
            self.runner = ptr::null_mut();
        }
    }
}
