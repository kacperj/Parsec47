use core::ffi::c_float;

use crate::gl::*;
use crate::renderer::*;

const LETTER_NUM: i32 = 42;

const TO_RIGHT: i32 = 0;
const TO_DOWN: i32 = 1;
const TO_LEFT: i32 = 2;
const TO_UP: i32 = 3;

static mut DISPLAY_LIST_IDX: GLuint = 0;
static mut COLOR_IDX: i32 = 0;

type Stroke = [c_float; 5];
const END: Stroke = [0.0, 0.0, 0.0, 0.0, 99999.0];

#[rustfmt::skip]
static SP_DATA: [[Stroke; 16]; 42] = [
    // 0
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.6, 0.55, 0.65, 0.3, 90.0], [0.6, 0.55, 0.65, 0.3, 90.0],
        [-0.6, -0.55, 0.65, 0.3, 90.0], [0.6, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // 1
    [
        [0.0, 0.55, 0.65, 0.3, 90.0],
        [0.0, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // 2
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // 3
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // 4
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // 5
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // 6
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // 7
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // 8
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END,
    ],
    // 9
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // A
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // B
    [
        [-0.1, 1.15, 0.45, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.45, 0.55, 0.65, 0.3, 90.0],
        [-0.1, 0.0, 0.45, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END,
    ],
    // C
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // D
    [
        [-0.1, 1.15, 0.45, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.45, 0.4, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // E
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // F
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // G
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [0.25, 0.0, 0.25, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // H
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // I
    [
        [0.0, 0.55, 0.65, 0.3, 90.0],
        [0.0, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // J
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.75, 0.25, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // K
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.45, 0.55, 0.65, 0.3, 90.0],
        [-0.1, 0.0, 0.45, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // L
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // M
    [
        [-0.3, 1.15, 0.25, 0.3, 0.0], [0.3, 1.15, 0.25, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, 0.55, 0.65, 0.3, 90.0],
        [0.0, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END,
    ],
    // N
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // O
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // P
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // Q
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        [0.2, -0.6, 0.45, 0.3, 60.0],
        END, END, END, END, END, END, END, END, END,
    ],
    // R
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.1, 0.0, 0.45, 0.3, 0.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.45, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END,
    ],
    // S
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [-0.65, 0.55, 0.65, 0.3, 90.0],
        [0.0, 0.0, 0.65, 0.3, 0.0],
        [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // T
    [
        [-0.4, 1.15, 0.45, 0.3, 0.0], [0.4, 1.15, 0.45, 0.3, 0.0],
        [0.0, 0.55, 0.65, 0.3, 90.0],
        [0.0, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // U
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // V
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.5, -0.55, 0.65, 0.3, 90.0], [0.5, -0.55, 0.65, 0.3, 90.0],
        [0.0, -1.15, 0.45, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END,
    ],
    // W
    [
        [-0.65, 0.55, 0.65, 0.3, 90.0], [0.65, 0.55, 0.65, 0.3, 90.0],
        [-0.65, -0.55, 0.65, 0.3, 90.0], [0.65, -0.55, 0.65, 0.3, 90.0],
        [-0.3, -1.15, 0.25, 0.3, 0.0], [0.3, -1.15, 0.25, 0.3, 0.0],
        [0.0, 0.55, 0.65, 0.3, 90.0],
        [0.0, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END,
    ],
    // X
    [
        [-0.4, 0.6, 0.85, 0.3, 240.0],
        [0.4, 0.6, 0.85, 0.3, 300.0],
        [-0.4, -0.6, 0.85, 0.3, 120.0],
        [0.4, -0.6, 0.85, 0.3, 60.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // Y
    [
        [-0.4, 0.6, 0.85, 0.3, 240.0],
        [0.4, 0.6, 0.85, 0.3, 300.0],
        [0.0, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // Z
    [
        [0.0, 1.15, 0.65, 0.3, 0.0],
        [0.35, 0.5, 0.65, 0.3, 300.0],
        [-0.35, -0.5, 0.65, 0.3, 120.0],
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // . (36)
    [
        [0.0, -1.15, 0.05, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // _ (37)
    [
        [0.0, -1.15, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // - (38)
    [
        [0.0, 0.0, 0.65, 0.3, 0.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // + (39)
    [
        [-0.4, 0.0, 0.45, 0.3, 0.0], [0.4, 0.0, 0.45, 0.3, 0.0],
        [0.0, 0.55, 0.65, 0.3, 90.0],
        [0.0, -0.55, 0.65, 0.3, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // ' (40)
    [
        [0.0, 1.0, 0.4, 0.2, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
    // '' (41)
    [
        [-0.19, 1.0, 0.4, 0.2, 90.0],
        [0.2, 1.0, 0.4, 0.2, 90.0],
        END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    ],
];

fn draw_glyph_box(
    x: c_float,
    y: c_float,
    width: c_float,
    height: c_float,
    r: c_float,
    g: c_float,
    b: c_float,
) {
    set_color(r, g, b, 0.5);
    draw_box_solid(x - width, y - height, width * 2.0, height * 2.0);
    set_color(r, g, b, 1.0);
    draw_box_line(x - width, y - height, width * 2.0, height * 2.0);
}

fn draw_letter_glyph(idx: usize, r: c_float, g: c_float, b: c_float) {
    let glyph = &SP_DATA[idx];
    for stroke in glyph.iter() {
        let deg = stroke[4] as i32;
        if deg > 99990 {
            break;
        }
        let x = stroke[0];
        let y = -stroke[1];
        let mut size = stroke[2] * 0.66;
        let mut length = stroke[3] * 0.6;
        let deg_mod = deg % 180;
        if deg_mod <= 45 || deg_mod > 135 {
            draw_glyph_box(x, y, size, length, r, g, b);
        } else {
            draw_glyph_box(x, y, length, size, r, g, b);
        }
    }
}

fn draw_letter_at(n: i32, x: c_float, y: c_float, s: c_float, d: c_float) {
    unsafe {
        glPushMatrix();
        glTranslatef(x, y, 0.0);
        glScalef(s, s, s);
        glRotatef(d, 0.0, 0.0, 1.0);
        glCallList(DISPLAY_LIST_IDX + n as GLuint + COLOR_IDX as GLuint);
        glPopMatrix();
    }
}

fn direction_to_angle(d: i32) -> c_float {
    match d {
        TO_RIGHT => 0.0,
        TO_DOWN => 90.0,
        TO_LEFT => 180.0,
        TO_UP => 270.0,
        _ => 0.0,
    }
}

fn char_to_glyph_index(c: u8) -> Option<i32> {
    match c {
        b'0'..=b'9' => Some((c - b'0') as i32),
        b'A'..=b'Z' => Some((c - b'A') as i32 + 10),
        b'a'..=b'z' => Some((c - b'a') as i32 + 10),
        b'.' => Some(36),
        b'-' => Some(38),
        b'+' => Some(39),
        b' ' => None,
        _ => Some(37),
    }
}

#[no_mangle]
pub extern "C" fn letter_render_create_display_lists() {
    unsafe {
        DISPLAY_LIST_IDX = glGenLists(LETTER_NUM as GLsizei * 2);
        for i in 0..LETTER_NUM {
            glNewList(DISPLAY_LIST_IDX + i as GLuint, GL_COMPILE);
            draw_letter_glyph(i as usize, 1.0, 1.0, 1.0);
            glEndList();
        }
        for i in 0..LETTER_NUM {
            glNewList(DISPLAY_LIST_IDX + LETTER_NUM as GLuint + i as GLuint, GL_COMPILE);
            draw_letter_glyph(i as usize, 1.0, 0.7, 0.7);
            glEndList();
        }
    }
}

#[no_mangle]
pub extern "C" fn letter_render_delete_display_lists() {
    unsafe {
        glDeleteLists(DISPLAY_LIST_IDX, LETTER_NUM as GLsizei * 2);
    }
}

pub fn letter_render_change_color(c: i32) {
    unsafe {
        COLOR_IDX = c * LETTER_NUM;
    }
}

#[no_mangle]
pub extern "C" fn letter_render_draw_string(
    ptr: *const u8,
    len: i32,
    lx: c_float,
    y: c_float,
    s: c_float,
    d: i32,
) {
    let ld = direction_to_angle(d);
    let mut x = lx;
    let mut y = y;
    let step = s * 1.7;

    for i in 0..len {
        let c = unsafe { *ptr.add(i as usize) };
        if let Some(idx) = char_to_glyph_index(c) {
            draw_letter_at(idx, x, y, s, ld);
        }
        match d {
            TO_RIGHT => x += step,
            TO_DOWN => y += step,
            TO_LEFT => x -= step,
            TO_UP => y -= step,
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn letter_render_draw_num(
    num: i32,
    lx: c_float,
    y: c_float,
    s: c_float,
    d: i32,
) {
    let ld = direction_to_angle(d);
    let mut x = lx;
    let mut y = y;
    let step = s * 1.7;
    let mut n = num;

    loop {
        draw_letter_at(n % 10, x, y, s, ld);
        match d {
            TO_RIGHT => x -= step,
            TO_DOWN => y -= step,
            TO_LEFT => x += step,
            TO_UP => y += step,
            _ => {}
        }
        n /= 10;
        if n <= 0 {
            break;
        }
    }
}
