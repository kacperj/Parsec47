use crate::letter_render::*;
use crate::renderer::*;
use crate::rendering::gl::*;
use crate::state::state_export::*;
use core::ffi::c_float;

#[no_mangle]
pub extern "C" fn renderer_set_color_params(r: c_float, g: c_float, b: c_float, a: c_float) {
    set_color_params(r, g, b, a);
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_solid(x: c_float, y: c_float, width: c_float, height: c_float) {
    draw_box_solid(x, y, width, height);
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_line(x: c_float, y: c_float, width: c_float, height: c_float) {
    draw_box_line(x, y, width, height);
}

fn draw_board(x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        glColor4f(0.0, 0.0, 0.0, 1.0);
    }
    draw_box_solid(
        x as c_float,
        y as c_float,
        width as c_float,
        height as c_float,
    );
}

#[no_mangle]
pub extern "C" fn renderer_draw_box(x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 {
        return;
    }
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color_params(1.0, 1.0, 1.0, 0.5);
    draw_box_solid(x, y, w, h);
    set_color_params(1.0, 1.0, 1.0, 1.0);
    draw_box_line(x, y, w, h);
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_outlined(x: i32, y: i32, w: i32, h: i32) {
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color_params(1.0, 1.0, 1.0, 1.0);
    draw_box_line(x, y, w, h);
    set_color_params(1.0, 1.0, 1.0, 0.5);
    draw_box_solid(x, y, w, h);
}

#[no_mangle]
pub extern "C" fn renderer_draw_box_light(x: i32, y: i32, w: i32, h: i32) {
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color_params(1.0, 1.0, 1.0, 0.7);
    draw_box_line(x, y, w, h);
    set_color_params(1.0, 1.0, 1.0, 0.3);
    draw_box_solid(x, y, w, h);
}

fn renderer_draw_left(left: i32) {
    if left < 0 {
        return;
    }
    let text = b"LEFT";
    letter_render_draw_string(text.as_ptr(), 4, 520.0, 260.0, 25.0, 1);
    letter_render_change_color(1);
    letter_render_draw_num(left, 520.0, 450.0, 25.0, 1);
    letter_render_change_color(0);
}

#[no_mangle]
pub extern "C" fn renderer_draw_side_info(parsec: i32) {
    renderer_draw_side_boards();
    renderer_draw_score();
    renderer_draw_left(life_get());
    renderer_draw_parsec(parsec);
}

#[no_mangle]
pub extern "C" fn renderer_draw_score() {
    letter_render_draw_num(score_get(), 120.0, 28.0, 25.0, 3);
    letter_render_draw_num(get_bonus_state(), 24.0, 20.0, 12.0, 3);
}

fn renderer_draw_parsec(parsec: i32) {
    let y = if parsec < 10 {
        26.0
    } else if parsec < 100 {
        68.0
    } else {
        110.0
    };
    letter_render_draw_num(parsec, 600.0, y, 25.0, 1);
}

static TITLE_BMP: &[u8] = include_bytes!("../../assets/images/title.bmp");
static mut TITLE_TEXTURE: GLuint = 0;

fn load_bmp_from_bytes(data: &[u8]) -> (i32, i32, *const u8) {
    let pixel_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
    let width = i32::from_le_bytes([data[18], data[19], data[20], data[21]]);
    let height = i32::from_le_bytes([data[22], data[23], data[24], data[25]]);
    (width, height.abs(), data[pixel_offset..].as_ptr())
}

#[no_mangle]
pub extern "C" fn renderer_title_texture_init() {
    let (w, h, pixels) = load_bmp_from_bytes(TITLE_BMP);
    unsafe {
        glGenTextures(1, &mut TITLE_TEXTURE);
        glBindTexture(GL_TEXTURE_2D, TITLE_TEXTURE);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            3,
            w,
            h,
            0,
            GL_BGR,
            GL_UNSIGNED_BYTE,
            pixels,
        );
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    }
}

#[no_mangle]
pub extern "C" fn renderer_title_texture_delete() {
    unsafe {
        glDeleteTextures(1, &TITLE_TEXTURE);
        TITLE_TEXTURE = 0;
    }
}

#[no_mangle]
pub extern "C" fn renderer_draw_title_board() {
    unsafe {
        glEnable(GL_TEXTURE_2D);
        glBindTexture(GL_TEXTURE_2D, TITLE_TEXTURE);
    }
    set_color_params(1.0, 1.0, 1.0, 1.0);
    unsafe {
        glBegin(GL_TRIANGLE_FAN);
        glTexCoord2f(0.0, 1.0);
        glVertex3f(180.0, 20.0, 0.0);
        glTexCoord2f(1.0, 1.0);
        glVertex3f(308.0, 20.0, 0.0);
        glTexCoord2f(1.0, 0.0);
        glVertex3f(308.0, 148.0, 0.0);
        glTexCoord2f(0.0, 0.0);
        glVertex3f(180.0, 148.0, 0.0);
        glEnd();
        glDisable(GL_TEXTURE_2D);
    }
}

#[no_mangle]
pub extern "C" fn renderer_draw_gameover_status(
    parsec: i32,
    cnt: i32,
) {
    renderer_draw_side_info(parsec);
    if cnt > 64 {
        letter_render_draw_string(b"GAME OVER".as_ptr(), 9, 220.0, 200.0, 15.0, 0);
    }
}

#[no_mangle]
pub extern "C" fn renderer_draw_pause_status(
    parsec: i32,
    pause_cnt: i32,
) {
    renderer_draw_side_info(parsec);
    if (pause_cnt % 60) < 30 {
        letter_render_draw_string(b"PAUSE".as_ptr(), 5, 280.0, 220.0, 12.0, 0);
    }
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
