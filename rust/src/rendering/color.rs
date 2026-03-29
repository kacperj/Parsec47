use core::ffi::c_float;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Color {
    pub r: c_float,
    pub g: c_float,
    pub b: c_float,
    pub a: c_float,
}
