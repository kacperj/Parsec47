use core::ffi::c_float;
use std::ops::{Add, Mul, Neg, Sub};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: c_float,
    pub y: c_float,
}

impl Add for Vector2 {
    type Output = Vector2;
    fn add(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, rhs: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f32) -> Vector2 {
        Vector2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Neg for Vector2 {
    type Output = Vector2;
    fn neg(self) -> Vector2 {
        Vector2 {
            x: -self.x,
            y: -self.y,
        }
    }
}
