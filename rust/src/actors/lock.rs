use crate::core::vector::Vector2;
use crate::renderer::{draw_box_retro, draw_line_retro_with_z};
use crate::rendering::color::Color;
use core::ffi::{c_float, c_int};

const LENGTH: usize = 12;
const LOCK_ANIM_DURATION: f32 = 8.0;

const LOCK_COLOR: Color = Color {
    r: 1.0,
    g: 0.8,
    b: 0.5,
    a: 1.0,
};

// Must match the state enum in src/abagames/p47/Lock.d.
const STATE_LOCKING: c_int = 2;
const STATE_LOCKED: c_int = 3;
const STATE_FIRED: c_int = 4;
const STATE_HIT: c_int = 5;
const STATE_CANCELED: c_int = 6;

fn draw_lock_marker(center_x: f32, center_y: f32, r: f32, mut d: f32, retro: f32, retro_size: f32) {
    for _ in 0..3 {
        draw_box_retro(
            center_x + d.sin() * r,
            center_y + d.cos() * r,
            0.2,
            1.0,
            d + 3.14 / 2.0,
            LOCK_COLOR,
            retro,
            retro_size,
        );
        d += 6.28 / 3.0;
    }
}

#[no_mangle]
pub extern "C" fn lock_draw(
    state: c_int,
    lock_anim_progress: c_int,
    locked_pos_x: c_float,
    locked_pos_y: c_float,
    laser_trace: *const Vector2,
) {
    match state {
        STATE_LOCKING => {
            let animation_progress = LOCK_ANIM_DURATION - lock_anim_progress as f32;
            let y = locked_pos_y - animation_progress * 0.5;
            let d = animation_progress * 0.1;
            let r = animation_progress * 0.5 + 0.8;
            let retro = animation_progress / LOCK_ANIM_DURATION;
            draw_lock_marker(locked_pos_x, y, r, d, retro, 0.2);
        }
        STATE_LOCKED | STATE_FIRED | STATE_CANCELED | STATE_HIT => {
            draw_lock_marker(locked_pos_x, locked_pos_y, 0.8, 0.0, 0.0, 0.2);

            let trace = unsafe { std::slice::from_raw_parts(laser_trace, LENGTH) };

            let mut r = lock_anim_progress as f32 * 0.1;
            for i in 0..LENGTH - 1 {
                let rr = r.clamp(0.0, 1.0);
                draw_line_retro_with_z(
                    trace[i].x, trace[i].y,
                    trace[i + 1].x, trace[i + 1].y,
                    0.0, rr, 0.33, LOCK_COLOR,
                );
                r -= 0.1;
            }
        }
        _ => {}
    }
}
