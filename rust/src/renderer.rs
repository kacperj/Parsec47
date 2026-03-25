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
pub extern "C" fn mix_retro_color(retro: c_float, r: c_float, g: c_float, b: c_float, a: c_float) {
    let cf = (1.0 - retro) * 0.5;
    let mut mr = r + (1.0 - r) * cf;
    let mut mg = g + (1.0 - g) * cf;
    let mut mb = b + (1.0 - b) * cf;
    let mut ma = a * (cf + 0.5);
    if crate::rand::rand_next_int(7) == 0 {
        mr = (mr * 1.5).min(1.0);
        mg = (mg * 1.5).min(1.0);
        mb = (mb * 1.5).min(1.0);
        ma = (ma * 1.5).min(1.0);
    }
    set_color(mr, mg, mb, ma);
}

fn draw_square_retro(x: c_float, y: c_float, ds: c_float, z: c_float) {
    let ds2 = ds / 2.0;
    unsafe {
        glVertex3f(x - ds2, y - ds2, z);
        glVertex3f(x + ds2, y - ds2, z);
        glVertex3f(x + ds2, y + ds2, z);
        glVertex3f(x - ds2, y + ds2, z);
    }
}

#[no_mangle]
pub extern "C" fn draw_line_retro_with_z(
    x1: c_float, y1: c_float, x2: c_float, y2: c_float, z: c_float,
    retro: c_float, retro_size: c_float,
    r: c_float, g: c_float, b: c_float, a: c_float,
) {
    mix_retro_color(retro, r, g, b, a);

    if retro < 0.2 {
        unsafe {
            glBegin(GL_LINES);
            glVertex3f(x1, y1, z);
            glVertex3f(x2, y2, z);
            glEnd();
        }
        return;
    }

    let ds = retro_size * retro;
    let lx = (x2 - x1).abs();
    let ly = (y2 - y1).abs();

    unsafe {
        glBegin(GL_QUADS);
        if lx < ly {
            let n = (ly / ds) as i32;
            if n > 0 {
                let xo = (x2 - x1) / n as c_float;
                let mut xos: c_float = 0.0;
                let yo = if y2 < y1 { -ds } else { ds };
                let mut x = x1;
                let mut y = y1;
                for _ in 0..=n {
                    if xos >= ds {
                        x += ds;
                        xos -= ds;
                    } else if xos <= -ds {
                        x -= ds;
                        xos += ds;
                    }
                    draw_square_retro(x, y, ds, z);
                    xos += xo;
                    y += yo;
                }
            }
        } else {
            let n = (lx / ds) as i32;
            if n > 0 {
                let yo = (y2 - y1) / n as c_float;
                let mut yos: c_float = 0.0;
                let xo = if x2 < x1 { -ds } else { ds };
                let mut x = x1;
                let mut y = y1;
                for _ in 0..=n {
                    if yos >= ds {
                        y += ds;
                        yos -= ds;
                    } else if yos <= -ds {
                        y -= ds;
                        yos += ds;
                    }
                    draw_square_retro(x, y, ds, z);
                    x += xo;
                    yos += yo;
                }
            }
        }
        glEnd();
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