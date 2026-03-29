use crate::rendering::gl::*;
use core::f32::consts::PI;

const RING_POS_NUM: usize = 16;
const RING_DEG: f32 = PI / 3.0 / (RING_POS_NUM as f32 / 2.0 + 0.5);
const RING_RADIUS: f32 = 10.0;
const RING_SIZE: f32 = 0.5;

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

#[no_mangle]
pub extern "C" fn field_create_ring_display_list() -> GLuint {
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
        idx
    }
}
