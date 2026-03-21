#![no_std]

mod letter_render;

use core::ffi::c_float;

pub(crate) type GLenum = u32;
pub(crate) type GLint = i32;
pub(crate) type GLuint = u32;
pub(crate) type GLsizei = i32;

pub(crate) const GL_TRIANGLE_FAN: GLenum = 0x0006;
pub(crate) const GL_LINE_LOOP: GLenum = 0x0002;
pub(crate) const GL_COMPILE: GLenum = 0x1300;
pub(crate) const GL_BLEND: GLenum = 0x0BE2;

#[link(name = "opengl32")]
extern "system" {
    pub(crate) fn glColor4f(red: c_float, green: c_float, blue: c_float, alpha: c_float);
    pub(crate) fn glBegin(mode: GLenum);
    pub(crate) fn glEnd();
    pub(crate) fn glVertex3f(x: c_float, y: c_float, z: c_float);
    pub(crate) fn glPushMatrix();
    pub(crate) fn glPopMatrix();
    pub(crate) fn glTranslatef(x: c_float, y: c_float, z: c_float);
    pub(crate) fn glScalef(x: c_float, y: c_float, z: c_float);
    pub(crate) fn glRotatef(angle: c_float, x: c_float, y: c_float, z: c_float);
    pub(crate) fn glCallList(list: GLuint);
    pub(crate) fn glGenLists(range: GLsizei) -> GLuint;
    pub(crate) fn glNewList(list: GLuint, mode: GLenum);
    pub(crate) fn glEndList();
    pub(crate) fn glDeleteLists(list: GLuint, range: GLsizei);
    pub(crate) fn glEnable(cap: GLenum);
    pub(crate) fn glDisable(cap: GLenum);
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

pub(crate) fn set_color(r: c_float, g: c_float, b: c_float, a: c_float) {
    unsafe {
        glColor4f(r * BRIGHTNESS, g * BRIGHTNESS, b * BRIGHTNESS, a);
    }
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_solid(x: c_float, y: c_float, width: c_float, height: c_float) {
    draw_box_solid(x, y, width, height);
}

pub(crate) fn draw_box_solid(x: c_float, y: c_float, width: c_float, height: c_float) {
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
    draw_box_line(x, y, width, height);
}

pub(crate) fn draw_box_line(x: c_float, y: c_float, width: c_float, height: c_float) {
    unsafe {
        glBegin(GL_LINE_LOOP);
        glVertex3f(x, y, 0.0);
        glVertex3f(x + width, y, 0.0);
        glVertex3f(x + width, y + height, 0.0);
        glVertex3f(x, y + height, 0.0);
        glEnd();
    }
}

fn draw_board(x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        glColor4f(0.0, 0.0, 0.0, 1.0);
    }
    draw_box_solid(x as c_float, y as c_float, width as c_float, height as c_float);
}

#[no_mangle]
pub extern "C" fn renderer_draw_box(x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 {
        return;
    }
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color(1.0, 1.0, 1.0, 0.5);
    draw_box_solid(x, y, w, h);
    set_color(1.0, 1.0, 1.0, 1.0);
    draw_box_line(x, y, w, h);
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_outlined(x: i32, y: i32, w: i32, h: i32) {
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color(1.0, 1.0, 1.0, 1.0);
    draw_box_line(x, y, w, h);
    set_color(1.0, 1.0, 1.0, 0.5);
    draw_box_solid(x, y, w, h);
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_light(x: i32, y: i32, w: i32, h: i32) {
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color(1.0, 1.0, 1.0, 0.7);
    draw_box_line(x, y, w, h);
    set_color(1.0, 1.0, 1.0, 0.3);
    draw_box_solid(x, y, w, h);
}

fn renderer_draw_left(left: i32) {
    if left < 0 {
        return;
    }
    let text = b"LEFT";
    letter_render::letter_render_draw_string(text.as_ptr(), 4, 520.0, 260.0, 25.0, 1);
    letter_render::letter_render_change_color(1);
    letter_render::letter_render_draw_num(left, 520.0, 450.0, 25.0, 1);
    letter_render::letter_render_change_color(0);
}

#[no_mangle]
pub extern "C" fn renderer_draw_side_info(score: i32, bonus_score: i32, left: i32, parsec: i32) {
    renderer_draw_side_boards();
    renderer_draw_score(score, bonus_score);
    renderer_draw_left(left);
    renderer_draw_parsec(parsec);
}

#[no_mangle]
pub extern "C" fn renderer_draw_score(score: i32, bonus_score: i32) {
    letter_render::letter_render_draw_num(score, 120.0, 28.0, 25.0, 3);
    letter_render::letter_render_draw_num(bonus_score, 24.0, 20.0, 12.0, 3);
}

fn renderer_draw_parsec(parsec: i32) {
    let y = if parsec < 10 {
        26.0
    } else if parsec < 100 {
        68.0
    } else {
        110.0
    };
    letter_render::letter_render_draw_num(parsec, 600.0, y, 25.0, 1);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn renderer_draw_side_boards() {
    unsafe {
        glDisable(GL_BLEND);
    }
    draw_board(0, 0, 160, 480);
    draw_board(480, 0, 160, 480);
    unsafe {
        glEnable(GL_BLEND);
    }
}