use crate::rendering::gl::*;
use core::ffi::{c_float, c_int};
use core::ptr;

const LUMINOUS_TEXTURE_WIDTH: i32 = 64;
const LUMINOUS_TEXTURE_HEIGHT: i32 = 64;

struct LuminousScreen {
    texture: GLuint,
    screen_width: i32,
    screen_height: i32,
    luminous: c_float,
}

static mut STATE: Option<LuminousScreen> = None;

pub fn luminous_screen_init(luminous: c_float, width: c_int, height: c_int) {
    let texture = unsafe {
        let mut tex: GLuint = 0;
        let pixel_count =
            (LUMINOUS_TEXTURE_WIDTH * LUMINOUS_TEXTURE_HEIGHT * 4) as usize;
        let mut data = vec![0u8; pixel_count];
        glGenTextures(1, &mut tex);
        glBindTexture(GL_TEXTURE_2D, tex);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            4,
            LUMINOUS_TEXTURE_WIDTH,
            LUMINOUS_TEXTURE_HEIGHT,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            data.as_mut_ptr(),
        );
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        tex
    };
    unsafe {
        STATE = Some(LuminousScreen {
            texture,
            screen_width: width,
            screen_height: height,
            luminous,
        });
    }
}

pub fn luminous_screen_close() {
    unsafe {
        if let Some(ref s) = STATE {
            glDeleteTextures(1, &s.texture);
        }
        STATE = None;
    }
}

pub fn luminous_screen_resized(width: c_int, height: c_int) {
    unsafe {
        if let Some(ref mut s) = STATE {
            s.screen_width = width;
            s.screen_height = height;
        }
    }
}

pub fn luminous_screen_start_render_to_texture() {
    unsafe {
        glViewport(0, 0, LUMINOUS_TEXTURE_WIDTH, LUMINOUS_TEXTURE_HEIGHT);
    }
}

pub fn luminous_screen_end_render_to_texture() {
    unsafe {
        if let Some(ref s) = STATE {
            glBindTexture(GL_TEXTURE_2D, s.texture);
            glCopyTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA,
                0,
                0,
                LUMINOUS_TEXTURE_WIDTH,
                LUMINOUS_TEXTURE_HEIGHT,
                0,
            );
            glViewport(0, 0, s.screen_width, s.screen_height);
        }
    }
}

static LM_OFS: [[i32; 2]; 5] = [[0, 0], [1, 0], [-1, 0], [0, 1], [0, -1]];
const LM_OFS_BS: c_float = 5.0;

pub fn luminous_screen_draw() {
    unsafe {
        if let Some(ref s) = STATE {
            let sw = s.screen_width as c_float;
            let sh = s.screen_height as c_float;
            let lm = s.luminous;

            glEnable(GL_TEXTURE_2D);
            glBindTexture(GL_TEXTURE_2D, s.texture);

            // switch to ortho projection covering screen pixels
            glMatrixMode(GL_PROJECTION);
            glPushMatrix();
            glLoadIdentity();
            glOrtho(0.0, sw as f64, sh as f64, 0.0, -1.0, 1.0);
            glMatrixMode(GL_MODELVIEW);
            glPushMatrix();
            glLoadIdentity();

            glColor4f(1.0, 0.8, 0.9, lm);
            glBegin(GL_QUADS);
            for i in 0..5 {
                let ox = LM_OFS[i][0] as c_float * LM_OFS_BS;
                let oy = LM_OFS[i][1] as c_float * LM_OFS_BS;
                glTexCoord2f(0.0, 1.0);
                glVertex2f(ox, oy);
                glTexCoord2f(0.0, 0.0);
                glVertex2f(ox, sh + oy);
                glTexCoord2f(1.0, 0.0);
                glVertex2f(sw + ox, sh + oy);
                glTexCoord2f(1.0, 1.0);
                glVertex2f(sw + ox, oy);
            }
            glEnd();

            glMatrixMode(GL_PROJECTION);
            glPopMatrix();
            glMatrixMode(GL_MODELVIEW);
            glPopMatrix();

            glDisable(GL_TEXTURE_2D);
        }
    }
}
