use core::ffi::c_float;
use crate::gl::*;

pub fn draw_box_solid(x: c_float, y: c_float, width: c_float, height: c_float) {
    unsafe {
        glBegin(GL_TRIANGLE_FAN);
        glVertex3f(x, y, 0.0);
        glVertex3f(x + width, y, 0.0);
        glVertex3f(x + width, y + height, 0.0);
        glVertex3f(x, y + height, 0.0);
        glEnd();
    }
}

pub fn draw_box_line(x: c_float, y: c_float, width: c_float, height: c_float) {
    unsafe {
        glBegin(GL_LINE_LOOP);
        glVertex3f(x, y, 0.0);
        glVertex3f(x + width, y, 0.0);
        glVertex3f(x + width, y + height, 0.0);
        glVertex3f(x, y + height, 0.0);
        glEnd();
    }
}

static mut BRIGHTNESS: c_float = 1.0;

pub fn set_color(r: c_float, g: c_float, b: c_float, a: c_float) {
    unsafe {
        glColor4f(r * BRIGHTNESS, g * BRIGHTNESS, b * BRIGHTNESS, a);
    }
}

#[no_mangle]
pub extern "C" fn renderer_set_brightness(b: c_float) {
    unsafe {
        BRIGHTNESS = b;
    }
}

#[no_mangle]
pub extern "C" fn renderer_get_brightness() -> c_float {
    unsafe { BRIGHTNESS }
}