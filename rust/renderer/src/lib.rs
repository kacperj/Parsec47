#![no_std]

use core::ffi::c_float;

type GLenum = u32;

const GL_TRIANGLE_FAN: GLenum = 0x0006;
const GL_LINE_LOOP: GLenum = 0x0002;

#[link(name = "opengl32")]
extern "system" {
    fn glColor4f(red: c_float, green: c_float, blue: c_float, alpha: c_float);
    fn glBegin(mode: GLenum);
    fn glEnd();
    fn glVertex3f(x: c_float, y: c_float, z: c_float);
}

static mut BRIGHTNESS: c_float = 1.0;

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

#[no_mangle]
pub extern "C" fn renderer_set_color(r: c_float, g: c_float, b: c_float, a: c_float) {
    unsafe {
        glColor4f(r * BRIGHTNESS, g * BRIGHTNESS, b * BRIGHTNESS, a);
    }
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_solid(x: c_float, y: c_float, width: c_float, height: c_float) {
    unsafe {
        glBegin(GL_TRIANGLE_FAN);
        glVertex3f(x, y, 0.0);
        glVertex3f(x + width, y, 0.0);
        glVertex3f(x + width, y + height, 0.0);
        glVertex3f(x, y + height, 0.0);
        glEnd();
    }
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_line(x: c_float, y: c_float, width: c_float, height: c_float) {
    unsafe {
        glBegin(GL_LINE_LOOP);
        glVertex3f(x, y, 0.0);
        glVertex3f(x + width, y, 0.0);
        glVertex3f(x + width, y + height, 0.0);
        glVertex3f(x, y + height, 0.0);
        glEnd();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
