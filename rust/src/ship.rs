use crate::renderer::{draw_box_line, draw_box_solid, set_color};
use crate::rendering::color::Color;
use crate::rendering::gl::*;
use core::ffi::c_float;

static mut SHIP_POS_X: c_float = 0.0;
static mut SHIP_POS_Y: c_float = 0.0;
static mut SHIP_CNT: i32 = 0;
static mut SHIP_DISPLAY_LIST_IDX: GLuint = 0;

#[no_mangle]
pub extern "C" fn ship_set_pos(x: c_float, y: c_float) {
    unsafe {
        SHIP_POS_X = x;
        SHIP_POS_Y = y;
    }
}

#[no_mangle]
pub extern "C" fn ship_get_pos_x() -> c_float {
    unsafe { SHIP_POS_X }
}

#[no_mangle]
pub extern "C" fn ship_get_pos_y() -> c_float {
    unsafe { SHIP_POS_Y }
}

#[no_mangle]
pub extern "C" fn ship_set_cnt(cnt: i32) {
    unsafe { SHIP_CNT = cnt; }
}

pub fn ship_get_cnt() -> i32 {
    unsafe { SHIP_CNT }
}

#[no_mangle]
pub extern "C" fn ship_create_display_lists() {
    unsafe {
        SHIP_DISPLAY_LIST_IDX = glGenLists(3);

        // List 0: wing segment
        glNewList(SHIP_DISPLAY_LIST_IDX, GL_COMPILE);
        set_color(Color { r: 0.5, g: 1.0, b: 0.5, a: 0.2 });
        draw_box_solid(-0.1, -0.5, 0.2, 1.0);
        set_color(Color { r: 0.5, g: 1.0, b: 0.5, a: 0.4 });
        draw_box_line(-0.1, -0.5, 0.2, 1.0);
        glEndList();

        // List 1: body core
        glNewList(SHIP_DISPLAY_LIST_IDX + 1, GL_COMPILE);
        set_color(Color { r: 1.0, g: 0.2, b: 0.2, a: 1.0 });
        draw_box_solid(-0.2, -0.2, 0.4, 0.4);
        set_color(Color { r: 1.0, g: 0.5, b: 0.5, a: 1.0 });
        draw_box_line(-0.2, -0.2, 0.4, 0.4);
        glEndList();

        // List 2: engine thruster
        glNewList(SHIP_DISPLAY_LIST_IDX + 2, GL_COMPILE);
        set_color(Color { r: 0.7, g: 1.0, b: 0.5, a: 0.3 });
        draw_box_solid(-0.15, -0.3, 0.3, 0.6);
        set_color(Color { r: 0.7, g: 1.0, b: 0.5, a: 0.6 });
        draw_box_line(-0.15, -0.3, 0.3, 0.6);
        glEndList();
    }
}

#[no_mangle]
pub extern "C" fn ship_draw(cnt: i32, bank: c_float, fire_wide_deg: c_float, ttl_cnt: i32) {
    const INVINCIBLE_CNT: i32 = 228;
    if cnt < -INVINCIBLE_CNT || (cnt < 0 && (-cnt % 32) < 16) {
        return;
    }
    let pos_x = unsafe { SHIP_POS_X };
    let pos_y = unsafe { SHIP_POS_Y };
    let dl = unsafe { SHIP_DISPLAY_LIST_IDX };
    unsafe {
        // Left wing assembly
        glPushMatrix();
        glTranslatef(pos_x, pos_y, 0.0);
        glCallList(dl + 1);
        glRotatef(bank, 0.0, 1.0, 0.0);
        glTranslatef(-0.5, 0.0, 0.0);
        glCallList(dl);
        glTranslatef(0.2, 0.3, 0.2);
        glCallList(dl);
        glTranslatef(0.0, 0.0, -0.4);
        glCallList(dl);
        glPopMatrix();
        // Right wing assembly
        glPushMatrix();
        glTranslatef(pos_x, pos_y, 0.0);
        glRotatef(bank, 0.0, 1.0, 0.0);
        glTranslatef(0.5, 0.0, 0.0);
        glCallList(dl);
        glTranslatef(-0.2, 0.3, 0.2);
        glCallList(dl);
        glTranslatef(0.0, 0.0, -0.4);
        glCallList(dl);
        glPopMatrix();
        // Engine thrusters (6 per side)
        for i in 0..6i32 {
            glPushMatrix();
            glTranslatef(pos_x - 0.7, pos_y - 0.3, 0.0);
            glRotatef(bank, 0.0, 1.0, 0.0);
            glRotatef(180.0 / 2.0 - fire_wide_deg * 100.0, 0.0, 0.0, 1.0);
            glRotatef(i as c_float * 180.0 / 3.0 - ttl_cnt as c_float * 4.0, 1.0, 0.0, 0.0);
            glTranslatef(0.0, 0.0, 0.7);
            glCallList(dl + 2);
            glPopMatrix();
            glPushMatrix();
            glTranslatef(pos_x + 0.7, pos_y - 0.3, 0.0);
            glRotatef(bank, 0.0, 1.0, 0.0);
            glRotatef(-180.0 / 2.0 + fire_wide_deg * 100.0, 0.0, 0.0, 1.0);
            glRotatef(i as c_float * 180.0 / 3.0 - ttl_cnt as c_float * 4.0, 1.0, 0.0, 0.0);
            glTranslatef(0.0, 0.0, 0.7);
            glCallList(dl + 2);
            glPopMatrix();
        }
    }
}
