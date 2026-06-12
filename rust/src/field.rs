use crate::collision::{check_hit, check_hit_with_space, CollisionBox};
use crate::renderer::set_color;
use crate::rendering::color::Color;
use crate::rendering::gl::*;
use core::f32::consts::PI;
use core::ffi::c_int;

const RING_POS_NUM: usize = 16;
const RING_DEG: f32 = PI / 3.0 / (RING_POS_NUM as f32 / 2.0 + 0.5);
const RING_RADIUS: f32 = 10.0;
const RING_SIZE: f32 = 0.5;

const RING_NUM: usize = 16;
const RING_ANGLE_INT: f32 = 10.0;

const MODE_ROLL: c_int = 0;
const MODE_LOCK: c_int = 1;

struct FieldState {
    collision_box: CollisionBox,
    aim_z: f32,
    aim_speed: f32,
    roll: f32,
    yaw: f32,
    z: f32,
    speed: f32,
    yaw_y_base: f32,
    yaw_z_base: f32,
    aim_yaw_y_base: f32,
    aim_yaw_z_base: f32,
    color: Color,
}

static mut FIELD: FieldState = FieldState {
    collision_box: CollisionBox {
        x1: 0.0,
        y1: 0.0,
        x2: 0.0,
        y2: 0.0,
    },
    aim_z: 10.0,
    aim_speed: 0.1,
    roll: 0.0,
    yaw: 0.0,
    z: 10.0,
    speed: 0.1,
    yaw_y_base: 0.0,
    yaw_z_base: 0.0,
    aim_yaw_y_base: 0.0,
    aim_yaw_z_base: 0.0,
    color: Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    },
};

static mut DISPLAY_LIST_IDX: GLuint = 0;

fn write_one_ring(ring_pos: &[(f32, f32); RING_POS_NUM]) {
    unsafe {
        glBegin(GL_LINE_STRIP);
        for i in 0..=(RING_POS_NUM / 2 - 2) {
            glVertex3f(ring_pos[i].0, RING_SIZE, ring_pos[i].1);
        }
        for i in (0..=(RING_POS_NUM / 2 - 2)).rev() {
            glVertex3f(ring_pos[i].0, -RING_SIZE, ring_pos[i].1);
        }
        glVertex3f(ring_pos[0].0, RING_SIZE, ring_pos[0].1);
        glEnd();

        glBegin(GL_LINE_STRIP);
        glVertex3f(
            ring_pos[RING_POS_NUM / 2 - 1].0,
            RING_SIZE,
            ring_pos[RING_POS_NUM / 2 - 1].1,
        );
        glVertex3f(
            ring_pos[RING_POS_NUM / 2].0,
            RING_SIZE,
            ring_pos[RING_POS_NUM / 2].1,
        );
        glVertex3f(
            ring_pos[RING_POS_NUM / 2].0,
            -RING_SIZE,
            ring_pos[RING_POS_NUM / 2].1,
        );
        glVertex3f(
            ring_pos[RING_POS_NUM / 2 - 1].0,
            -RING_SIZE,
            ring_pos[RING_POS_NUM / 2 - 1].1,
        );
        glVertex3f(
            ring_pos[RING_POS_NUM / 2 - 1].0,
            RING_SIZE,
            ring_pos[RING_POS_NUM / 2 - 1].1,
        );
        glEnd();

        glBegin(GL_LINE_STRIP);
        for i in (RING_POS_NUM / 2 + 1)..RING_POS_NUM {
            glVertex3f(ring_pos[i].0, RING_SIZE, ring_pos[i].1);
        }
        for i in ((RING_POS_NUM / 2 + 1)..RING_POS_NUM).rev() {
            glVertex3f(ring_pos[i].0, -RING_SIZE, ring_pos[i].1);
        }
        glVertex3f(
            ring_pos[RING_POS_NUM / 2 + 1].0,
            RING_SIZE,
            ring_pos[RING_POS_NUM / 2 + 1].1,
        );
        glEnd();
    }
}

pub fn field_create_ring_display_list() -> GLuint {
    let mut ring_pos = [(0.0f32, 0.0f32); RING_POS_NUM];
    let mut d = -RING_DEG * (RING_POS_NUM as f32 / 2.0 - 0.5);
    for pos in ring_pos.iter_mut() {
        *pos = (d.sin() * RING_RADIUS, d.cos() * RING_RADIUS);
        d += RING_DEG;
    }
    unsafe {
        let idx = glGenLists(1);
        glNewList(idx, GL_COMPILE);
        write_one_ring(&ring_pos);
        glEndList();
        DISPLAY_LIST_IDX = idx;
        idx
    }
}

pub fn field_init(half_width: f32, half_height: f32) {
    unsafe {
        FIELD.collision_box = CollisionBox::create_with_half_extents(half_width, half_height);
        FIELD.roll = 0.0;
        FIELD.yaw = 0.0;
        FIELD.z = 10.0;
        FIELD.aim_z = 10.0;
        FIELD.speed = 0.1;
        FIELD.aim_speed = 0.1;
        FIELD.yaw_y_base = 0.0;
        FIELD.yaw_z_base = 0.0;
    }
}

pub fn field_set_aim_z(z: f32) {
    unsafe {
        FIELD.aim_z = z;
    }
}

pub fn field_set_aim_speed(speed: f32) {
    unsafe {
        FIELD.aim_speed = speed;
    }
}

pub fn field_set_color(mode: c_int) {
    unsafe {
        FIELD.color = match mode {
            MODE_ROLL => Color {
                r: 0.2,
                g: 0.2,
                b: 0.7,
                a: 0.7,
            },
            MODE_LOCK => Color {
                r: 0.5,
                g: 0.3,
                b: 0.6,
                a: 0.7,
            },
            _ => return,
        };
    }
}

pub fn field_move() {
    unsafe {
        FIELD.roll += FIELD.speed;
        if FIELD.roll >= RING_ANGLE_INT {
            FIELD.roll -= RING_ANGLE_INT;
        }
        FIELD.yaw += FIELD.speed;
        FIELD.z += (FIELD.aim_z - FIELD.z) * 0.003;
        FIELD.speed += (FIELD.aim_speed - FIELD.speed) * 0.004;
        FIELD.yaw_y_base += (FIELD.aim_yaw_y_base - FIELD.yaw_y_base) * 0.002;
        FIELD.yaw_z_base += (FIELD.aim_yaw_z_base - FIELD.yaw_z_base) * 0.002;
    }
}

pub fn field_set_type(type_: c_int) {
    unsafe {
        match type_ {
            0 => {
                FIELD.aim_yaw_y_base = 30.0;
                FIELD.aim_yaw_z_base = 0.0;
            }
            1 => {
                FIELD.aim_yaw_y_base = 0.0;
                FIELD.aim_yaw_z_base = 20.0;
            }
            2 => {
                FIELD.aim_yaw_y_base = 50.0;
                FIELD.aim_yaw_z_base = 10.0;
            }
            3 => {
                FIELD.aim_yaw_y_base = 10.0;
                FIELD.aim_yaw_z_base = 30.0;
            }
            _ => {}
        }
    }
}

pub fn field_draw() {
    unsafe {
        set_color(FIELD.color);
        let mut d = -(RING_NUM as f32) * RING_ANGLE_INT / 2.0 + FIELD.roll;
        for _ in 0..RING_NUM {
            for j in 1..8 {
                let sc = j as f32 / 16.0 + 0.5;
                glPushMatrix();
                glTranslatef(0.0, 0.0, FIELD.z);
                glRotatef(d, 1.0, 0.0, 0.0);
                glRotatef(
                    (FIELD.yaw / 180.0 * PI).sin() * FIELD.yaw_y_base,
                    0.0,
                    1.0,
                    0.0,
                );
                glRotatef(
                    (FIELD.yaw / 180.0 * PI).sin() * FIELD.yaw_z_base,
                    0.0,
                    0.0,
                    1.0,
                );
                glScalef(1.0, 1.0, sc);
                glCallList(DISPLAY_LIST_IDX);
                glPopMatrix();
            }
            d += RING_ANGLE_INT;
        }
    }
}

pub fn field_check_hit(px: f32, py: f32) -> bool {
    unsafe { check_hit(FIELD.collision_box, px, py) }
}

pub fn field_check_hit_with_space(px: f32, py: f32, space: f32) -> bool {
    unsafe { check_hit_with_space(FIELD.collision_box, px, py, space) }
}

pub fn field_get_collision_box() -> CollisionBox {
    unsafe { FIELD.collision_box }
}
