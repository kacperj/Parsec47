use crate::renderer::{draw_line_retro_with_z, set_color};
use crate::rendering::color::Color;
use crate::rendering::gl::*;
use core::ffi::{c_float, c_int};

const BULLET_SHAPE_NUM: usize = 7;
const BULLET_COLOR_NUM: usize = 4;
const RETRO_CNT: c_float = 24.0;
const SHAPE_POINT_SIZE: c_float = 0.1;

static mut BULLET_DISPLAY_LIST_IDX: GLuint = 0;
const SHAPE_BASE_COLOR: Color = Color {
    r: 1.0,
    g: 0.9,
    b: 0.7,
    a: 0.55,
};

const BULLET_COLOR: [[c_float; 3]; BULLET_COLOR_NUM] = [
    [1.0, 0.0, 0.0],
    [0.2, 1.0, 0.4],
    [0.3, 0.3, 1.0],
    [1.0, 1.0, 0.0],
];

static SHAPE_POS: [&[(c_float, c_float)]; BULLET_SHAPE_NUM] = [
    &[(-0.5, -0.5), (0.5, -0.5), (0.0, 1.0)],
    &[(0.0, -1.0), (0.5, 0.0), (0.0, 1.0), (-0.5, 0.0)],
    &[(-0.25, -0.66), (0.25, -0.66), (0.25, 0.66), (-0.25, 0.66)],
    &[(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)],
    &[
        (-0.25, -0.5),
        (0.25, -0.5),
        (0.5, -0.25),
        (0.5, 0.25),
        (0.25, 0.5),
        (-0.25, 0.5),
        (-0.5, 0.25),
        (-0.5, -0.25),
    ],
    &[(-0.66, -0.46), (0.0, 0.86), (0.66, -0.46)],
    &[
        (-0.5, -0.5),
        (0.0, -0.5),
        (0.5, 0.0),
        (0.5, 0.5),
        (0.0, 0.5),
        (-0.5, 0.0),
    ],
];

pub fn bullet_actor_draw_retro(
    d: c_float,
    rt: c_float,
    bullet_size: c_float,
    shape: c_int,
    color: c_int,
) {
    let rgb = BULLET_COLOR[color as usize];
    let c = Color {
        r: rgb[0],
        g: rgb[1],
        b: rgb[2],
        a: 1.0,
    };
    let retro_size = 0.4 * bullet_size;
    let sin_d = d.sin();
    let cos_d = d.cos();
    let pts = SHAPE_POS[shape as usize];
    let mut prev_x: c_float = 0.0;
    let mut prev_y: c_float = 0.0;
    let mut fx: c_float = 0.0;
    let mut fy: c_float = 0.0;
    for (i, &(sx, sy)) in pts.iter().enumerate() {
        let tx = sx * bullet_size;
        let ty = sy * bullet_size;
        let x = tx * cos_d - ty * sin_d;
        let y = tx * sin_d + ty * cos_d;
        if i > 0 {
            draw_line_retro_with_z(prev_x, prev_y, x, y, 0.0, rt, retro_size, c);
        } else {
            fx = x;
            fy = y;
        }
        prev_x = x;
        prev_y = y;
    }
    draw_line_retro_with_z(prev_x, prev_y, fx, fy, 0.0, rt, retro_size, c);
}

pub fn bullet_actor_create_display_lists() {
    let size: c_float = 1.0;
    let base_idx = unsafe { glGenLists((BULLET_COLOR_NUM * (BULLET_SHAPE_NUM + 1)) as GLsizei) };
    unsafe {
        BULLET_DISPLAY_LIST_IDX = base_idx;
    }
    let mut idx: u32 = 0;
    for i in 0..BULLET_COLOR_NUM {
        let mut r = BULLET_COLOR[i][0];
        let mut g = BULLET_COLOR[i][1];
        let mut b = BULLET_COLOR[i][2];
        r += (1.0 - r) * 0.5;
        g += (1.0 - g) * 0.5;
        b += (1.0 - b) * 0.5;
        let outline = Color { r, g, b, a: 1.0 };
        for j in 0..(BULLET_SHAPE_NUM + 1) {
            unsafe {
                glNewList(base_idx + idx, GL_COMPILE);
                set_color(outline);
                match j {
                    0 => {
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(-SHAPE_POINT_SIZE, -SHAPE_POINT_SIZE, 0.0);
                        glVertex3f(SHAPE_POINT_SIZE, -SHAPE_POINT_SIZE, 0.0);
                        glVertex3f(SHAPE_POINT_SIZE, SHAPE_POINT_SIZE, 0.0);
                        glVertex3f(-SHAPE_POINT_SIZE, SHAPE_POINT_SIZE, 0.0);
                        glEnd();
                    }
                    1 => {
                        let sz = size / 2.0;
                        glDisable(GL_BLEND);
                        glBegin(GL_LINE_LOOP);
                        glVertex3f(-sz, -sz, 0.0);
                        glVertex3f(sz, -sz, 0.0);
                        glVertex3f(0.0, size, 0.0);
                        glEnd();
                        glEnable(GL_BLEND);
                        set_color(Color { r, g, b, a: 0.55 });
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(-sz, -sz, 0.0);
                        glVertex3f(sz, -sz, 0.0);
                        set_color(SHAPE_BASE_COLOR);
                        glVertex3f(0.0, size, 0.0);
                        glEnd();
                    }
                    2 => {
                        let sz = size / 2.0;
                        glDisable(GL_BLEND);
                        glBegin(GL_LINE_LOOP);
                        glVertex3f(0.0, -size, 0.0);
                        glVertex3f(sz, 0.0, 0.0);
                        glVertex3f(0.0, size, 0.0);
                        glVertex3f(-sz, 0.0, 0.0);
                        glEnd();
                        glEnable(GL_BLEND);
                        set_color(Color { r, g, b, a: 0.7 });
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(0.0, -size, 0.0);
                        glVertex3f(sz, 0.0, 0.0);
                        set_color(SHAPE_BASE_COLOR);
                        glVertex3f(0.0, size, 0.0);
                        glVertex3f(-sz, 0.0, 0.0);
                        glEnd();
                    }
                    3 => {
                        let sz = size / 4.0;
                        let sz2 = size / 3.0 * 2.0;
                        glDisable(GL_BLEND);
                        glBegin(GL_LINE_LOOP);
                        glVertex3f(-sz, -sz2, 0.0);
                        glVertex3f(sz, -sz2, 0.0);
                        glVertex3f(sz, sz2, 0.0);
                        glVertex3f(-sz, sz2, 0.0);
                        glEnd();
                        glEnable(GL_BLEND);
                        set_color(Color { r, g, b, a: 0.45 });
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(-sz, -sz2, 0.0);
                        glVertex3f(sz, -sz2, 0.0);
                        set_color(SHAPE_BASE_COLOR);
                        glVertex3f(sz, sz2, 0.0);
                        glVertex3f(-sz, sz2, 0.0);
                        glEnd();
                    }
                    4 => {
                        let sz = size / 2.0;
                        glDisable(GL_BLEND);
                        glBegin(GL_LINE_LOOP);
                        glVertex3f(-sz, -sz, 0.0);
                        glVertex3f(sz, -sz, 0.0);
                        glVertex3f(sz, sz, 0.0);
                        glVertex3f(-sz, sz, 0.0);
                        glEnd();
                        glEnable(GL_BLEND);
                        set_color(Color { r, g, b, a: 0.7 });
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(-sz, -sz, 0.0);
                        glVertex3f(sz, -sz, 0.0);
                        set_color(SHAPE_BASE_COLOR);
                        glVertex3f(sz, sz, 0.0);
                        glVertex3f(-sz, sz, 0.0);
                        glEnd();
                    }
                    5 => {
                        let sz = size / 2.0;
                        glDisable(GL_BLEND);
                        glBegin(GL_LINE_LOOP);
                        glVertex3f(-sz / 2.0, -sz, 0.0);
                        glVertex3f(sz / 2.0, -sz, 0.0);
                        glVertex3f(sz, -sz / 2.0, 0.0);
                        glVertex3f(sz, sz / 2.0, 0.0);
                        glVertex3f(sz / 2.0, sz, 0.0);
                        glVertex3f(-sz / 2.0, sz, 0.0);
                        glVertex3f(-sz, sz / 2.0, 0.0);
                        glVertex3f(-sz, -sz / 2.0, 0.0);
                        glEnd();
                        glEnable(GL_BLEND);
                        set_color(Color { r, g, b, a: 0.85 });
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(-sz / 2.0, -sz, 0.0);
                        glVertex3f(sz / 2.0, -sz, 0.0);
                        glVertex3f(sz, -sz / 2.0, 0.0);
                        glVertex3f(sz, sz / 2.0, 0.0);
                        set_color(SHAPE_BASE_COLOR);
                        glVertex3f(sz / 2.0, sz, 0.0);
                        glVertex3f(-sz / 2.0, sz, 0.0);
                        glVertex3f(-sz, sz / 2.0, 0.0);
                        glVertex3f(-sz, -sz / 2.0, 0.0);
                        glEnd();
                    }
                    6 => {
                        let sz = size * 2.0 / 3.0;
                        let sz2 = size / 5.0;
                        glDisable(GL_BLEND);
                        glBegin(GL_LINE_STRIP);
                        glVertex3f(-sz, -sz + sz2, 0.0);
                        glVertex3f(0.0, sz + sz2, 0.0);
                        glVertex3f(sz, -sz + sz2, 0.0);
                        glEnd();
                        glEnable(GL_BLEND);
                        set_color(Color { r, g, b, a: 0.55 });
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(-sz, -sz + sz2, 0.0);
                        glVertex3f(sz, -sz + sz2, 0.0);
                        set_color(SHAPE_BASE_COLOR);
                        glVertex3f(0.0, sz + sz2, 0.0);
                        glEnd();
                    }
                    7 => {
                        let sz = size / 2.0;
                        glDisable(GL_BLEND);
                        glBegin(GL_LINE_LOOP);
                        glVertex3f(-sz, -sz, 0.0);
                        glVertex3f(0.0, -sz, 0.0);
                        glVertex3f(sz, 0.0, 0.0);
                        glVertex3f(sz, sz, 0.0);
                        glVertex3f(0.0, sz, 0.0);
                        glVertex3f(-sz, 0.0, 0.0);
                        glEnd();
                        glEnable(GL_BLEND);
                        set_color(Color { r, g, b, a: 0.85 });
                        glBegin(GL_TRIANGLE_FAN);
                        glVertex3f(-sz, -sz, 0.0);
                        glVertex3f(0.0, -sz, 0.0);
                        glVertex3f(sz, 0.0, 0.0);
                        set_color(SHAPE_BASE_COLOR);
                        glVertex3f(sz, sz, 0.0);
                        glVertex3f(0.0, sz, 0.0);
                        glVertex3f(-sz, 0.0, 0.0);
                        glEnd();
                    }
                    _ => {}
                }
                glEndList();
            }
            idx += 1;
        }
    }
}

pub fn bullet_actor_draw(
    shape: c_int,
    color: c_int,
    deg: c_float,
    x_reverse: c_float,
    cnt: c_int,
    pos_x: c_float,
    pos_y: c_float,
    rt_cnt: c_float,
    bullet_size: c_float,
) {
    let d: c_float = match shape {
        0 | 2 | 5 => -deg * x_reverse,
        1 => cnt as c_float * 0.14,
        3 => cnt as c_float * 0.23,
        4 => cnt as c_float * 0.33,
        6 => cnt as c_float * 0.08,
        _ => 0.0,
    };
    unsafe {
        glPushMatrix();
        glTranslatef(pos_x, pos_y, 0.0);
        if rt_cnt >= RETRO_CNT {
            let di = BULLET_DISPLAY_LIST_IDX
                + (color as GLuint) * (BULLET_SHAPE_NUM as GLuint + 1);
            glCallList(di);
            glRotatef(d.to_degrees(), 0.0, 0.0, 1.0);
            glScalef(bullet_size, bullet_size, 1.0);
            glCallList(di + 1 + shape as GLuint);
        } else {
            let rt = 1.0 - rt_cnt / RETRO_CNT;
            bullet_actor_draw_retro(d, rt, bullet_size, shape, color);
        }
        glPopMatrix();
    }
}
