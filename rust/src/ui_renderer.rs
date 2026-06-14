use crate::letter_render::*;
use crate::prefs::*;
use crate::renderer::*;
use crate::rendering::gl::*;
use crate::state::state_export::*;
use core::ffi::c_float;

const DIFFICULTY_STR: [&[u8]; 5] = [b"PRACTICE", b"NORMAL", b"HARD", b"EXTREME", b"QUIT"];
const DIFFICULTY_SHORT_STR: [&[u8]; 5] = [b"P", b"N", b"H", b"E", b"Q"];
const MODE_STR: [&[u8]; 2] = [b"ROLL", b"LOCK"];
const TITLE_DIFFICULTY_NUM: i32 = 4;
const TITLE_VERTICAL_BOXES: i32 = TITLE_DIFFICULTY_NUM + 1;
const TITLE_BOX_COUNT: i32 = 16;
const TITLE_BOX_SMALL: i32 = 24;
const TITLE_SLOT_NUM: i32 = 10;
const BOX_LETTER_SCALE: c_float = 12.0;

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

pub fn renderer_draw_box(x: i32, y: i32, w: i32, h: i32) {
    if w <= 0 {
        return;
    }
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color_params(1.0, 1.0, 1.0, 0.5);
    draw_box_solid(x, y, w, h);
    set_color_params(1.0, 1.0, 1.0, 1.0);
    draw_box_line(x, y, w, h);
}

pub fn renderer_draw_box_outlined(x: i32, y: i32, w: i32, h: i32) {
    let (x, y, w, h) = (x as c_float, y as c_float, w as c_float, h as c_float);
    set_color_params(1.0, 1.0, 1.0, 1.0);
    draw_box_line(x, y, w, h);
    set_color_params(1.0, 1.0, 1.0, 0.5);
    draw_box_solid(x, y, w, h);
}

pub fn renderer_draw_box_light(x: i32, y: i32, w: i32, h: i32) {
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

pub fn renderer_draw_side_info(parsec: i32) {
    renderer_draw_side_boards();
    renderer_draw_score();
    renderer_draw_left(score_state().get_life());
    renderer_draw_parsec(parsec);
}

pub fn renderer_draw_score() {
    letter_render_draw_num(score_state().get_score(), 120.0, 28.0, 25.0, 3);
    letter_render_draw_num(score_state().get_bonus_score(), 24.0, 20.0, 12.0, 3);
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

pub fn renderer_title_texture_init() {
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

pub fn renderer_title_texture_delete() {
    unsafe {
        glDeleteTextures(1, &TITLE_TEXTURE);
        TITLE_TEXTURE = 0;
    }
}

pub fn renderer_draw_title_board() {
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

pub fn renderer_draw_gameover_status(
    parsec: i32,
    cnt: i32,
) {
    renderer_draw_side_info(parsec);
    if cnt > 64 {
        letter_render_draw_string(b"GAME OVER".as_ptr(), 9, 220.0, 200.0, 15.0, 0);
    }
}

// Pause-menu buttons, in cursor order.
pub const PAUSE_RESUME: i32 = 0;
pub const PAUSE_SURRENDER: i32 = 1;

fn renderer_draw_pause_button(letter: &[u8], label: &[u8], box_y: i32, selected: bool, box_cnt: i32) {
    const BOX_X: i32 = 230;

    if selected {
        // Same enlarging-box animation as the selected title box.
        let bs = (TITLE_BOX_COUNT - box_cnt) / 2;
        renderer_draw_box_outlined(
            BOX_X - bs,
            box_y - bs,
            TITLE_BOX_SMALL + bs * 2,
            TITLE_BOX_SMALL + bs * 2,
        );

        // Letters stay anchored to the base box, so the box grows around them.
        let tx = BOX_X as c_float;
        let ty = (box_y + TITLE_BOX_SMALL) as c_float - 12.0;
        letter_render_draw_string(label.as_ptr(), label.len() as i32, tx + 46.0, ty, BOX_LETTER_SCALE, TO_RIGHT);
        letter_render_draw_string(letter.as_ptr(), letter.len() as i32, tx + 14.0, ty, BOX_LETTER_SCALE, TO_RIGHT);
    } else {
        renderer_draw_box_light(BOX_X, box_y, TITLE_BOX_SMALL, TITLE_BOX_SMALL);
    }
}

pub fn renderer_draw_pause_status(
    parsec: i32,
    pause_cnt: i32,
    cursor: i32,
    box_cnt: i32,
) {
    renderer_draw_side_info(parsec);
    if (pause_cnt % 60) < 30 {
        letter_render_draw_string(b"PAUSE".as_ptr(), 5, 280.0, 220.0, 12.0, 0);
    }
    renderer_draw_pause_button(b"R", b"ESUME", 270, cursor == PAUSE_RESUME, box_cnt);
    renderer_draw_pause_button(b"S", b"URRENDER", 302, cursor == PAUSE_SURRENDER, box_cnt);
}

pub fn renderer_draw_side_boards() {
    unsafe {
        glDisable(GL_BLEND);
    }
    draw_board(0, 0, 160, 480);
    draw_board(480, 0, 160, 480);
    unsafe {
        glEnable(GL_BLEND);
    }
}

pub fn renderer_draw_title(cur_x: i32, cur_y: i32, mode: i32, box_cnt: i32) {
    let diff_label = DIFFICULTY_STR[cur_y as usize];
    let mode_label = MODE_STR[mode as usize];
    letter_render_draw_string(
        diff_label.as_ptr(),
        diff_label.len() as i32,
        (470 - diff_label.len() as i32 * 14) as c_float,
        150.0,
        10.0,
        TO_RIGHT,
    );
    letter_render_draw_string(
        mode_label.as_ptr(),
        mode_label.len() as i32,
        (470 - mode_label.len() as i32 * 14) as c_float,
        450.0,
        10.0,
        TO_RIGHT,
    );

    if cur_x > 0 {
        let text = b"START AT PARSEC";
        letter_render_draw_string(text.as_ptr(), text.len() as i32, 290.0, 180.0, 6.0, TO_RIGHT);
        letter_render_draw_num(
            prefs_get_start_parsec(mode, cur_y, cur_x),
            470.0,
            180.0,
            6.0,
            TO_RIGHT,
        );
    }

    if cur_y < TITLE_DIFFICULTY_NUM {
        letter_render_draw_num(
            prefs_get_hi_score(mode, cur_y, cur_x),
            470.0,
            210.0,
            10.0,
            TO_RIGHT,
        );
    }

    for y in 0..TITLE_VERTICAL_BOXES {
        let slots = prefs_get_slots(mode, y);
        for x in 0..slots {
            let sx = 180 + x * 28;
            let mut sy: i32 = 260 + y * 32;

            if y == TITLE_DIFFICULTY_NUM {
                sy += 15;
            }

            if x == cur_x && y == cur_y {
                let bs = (TITLE_BOX_COUNT - box_cnt) / 2;
                renderer_draw_box_outlined(
                    sx - bs,
                    sy - bs,
                    TITLE_BOX_SMALL + bs * 2,
                    TITLE_BOX_SMALL + bs * 2,
                );

                if x == 0 {
                    let short = DIFFICULTY_SHORT_STR[y as usize];
                    letter_render_draw_string(
                        short.as_ptr(),
                        short.len() as i32,
                        (sx + 13) as c_float,
                        (sy + 13) as c_float,
                        BOX_LETTER_SCALE,
                        TO_RIGHT,
                    );
                } else {
                    let short = DIFFICULTY_SHORT_STR[y as usize];
                    letter_render_draw_string(
                        short.as_ptr(),
                        short.len() as i32,
                        (sx + 4) as c_float,
                        (sy + 13) as c_float,
                        BOX_LETTER_SCALE,
                        TO_RIGHT,
                    );
                    if x >= TITLE_SLOT_NUM - 1 {
                        let xstr = b"X";
                        letter_render_draw_string(
                            xstr.as_ptr(),
                            1,
                            (sx + 21) as c_float,
                            (sy + 14) as c_float,
                            BOX_LETTER_SCALE,
                            TO_RIGHT,
                        );
                    } else {
                        letter_render_draw_num(
                            x,
                            (sx + 22) as c_float,
                            (sy + 13) as c_float,
                            BOX_LETTER_SCALE,
                            TO_RIGHT,
                        );
                    }
                }
            } else {
                renderer_draw_box_light(sx, sy, TITLE_BOX_SMALL, TITLE_BOX_SMALL);
            }
        }
    }
    renderer_draw_title_board();
}
