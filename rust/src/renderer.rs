use crate::core::rand::*;
use crate::rendering::color::*;
use crate::rendering::gl::*;
use core::ffi::{c_float, c_int};

#[no_mangle]
pub extern "C" fn create_enemy_color(variant: c_int) -> Color {
    let (r, g, b) = match variant {
        0 => (1.0, rand_next_float(0.7) + 0.3, rand_next_float(0.7) + 0.3),
        1 => (rand_next_float(0.7) + 0.3, 1.0, rand_next_float(0.7) + 0.3),
        2 => (rand_next_float(0.7) + 0.3, rand_next_float(0.7) + 0.3, 1.0),
        _ => (0.0, 0.0, 0.0),
    };
    Color { r, g, b, a: 1.0 }
}

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

pub fn set_color_params(r: c_float, g: c_float, b: c_float, a: c_float) {
    unsafe {
        glColor4f(r * BRIGHTNESS, g * BRIGHTNESS, b * BRIGHTNESS, a);
    }
}

pub fn set_color(color: Color) {
    unsafe {
        glColor4f(
            color.r * BRIGHTNESS,
            color.g * BRIGHTNESS,
            color.b * BRIGHTNESS,
            color.a,
        );
    }
}

#[no_mangle]
pub extern "C" fn mix_retro_color(retro: c_float, color: Color) {
    let cf = (1.0 - retro) * 0.5;
    let mut mr = color.r + (1.0 - color.r) * cf;
    let mut mg = color.g + (1.0 - color.g) * cf;
    let mut mb = color.b + (1.0 - color.b) * cf;
    let mut ma = color.a * (cf + 0.5);
    if rand_next_int(7) == 0 {
        mr = (mr * 1.5).min(1.0);
        mg = (mg * 1.5).min(1.0);
        mb = (mb * 1.5).min(1.0);
        ma = (ma * 1.5).min(1.0);
    }
    set_color_params(mr, mg, mb, ma);
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
    x1: c_float,
    y1: c_float,
    x2: c_float,
    y2: c_float,
    z: c_float,
    retro: c_float,
    retro_size: c_float,
    color: Color,
) {
    mix_retro_color(retro, color);

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
pub extern "C" fn draw_box_retro(
    center_x: c_float,
    center_y: c_float,
    width: c_float,
    height: c_float,
    deg: c_float,
    color: Color,
    retro: c_float,
    retro_size: c_float,
) {
    let w1 = width * deg.cos() - height * deg.sin();
    let h1 = width * deg.sin() + height * deg.cos();
    let w2 = -width * deg.cos() - height * deg.sin();
    let h2 = -width * deg.sin() + height * deg.cos();
    draw_line_retro_with_z(
        center_x + w2,
        center_y - h2,
        center_x + w1,
        center_y - h1,
        0.0,
        retro,
        retro_size,
        color,
    );
    draw_line_retro_with_z(
        center_x + w1,
        center_y - h1,
        center_x - w2,
        center_y + h2,
        0.0,
        retro,
        retro_size,
        color,
    );
    draw_line_retro_with_z(
        center_x - w2,
        center_y + h2,
        center_x - w1,
        center_y + h1,
        0.0,
        retro,
        retro_size,
        color,
    );
    draw_line_retro_with_z(
        center_x - w1,
        center_y + h1,
        center_x + w2,
        center_y - h2,
        0.0,
        retro,
        retro_size,
        color,
    );
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
