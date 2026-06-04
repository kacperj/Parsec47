use crate::core::rand::rand_next_signed_float;
use crate::core::vector::Vector2;
use crate::rendering::gl::glTranslatef;

static mut SCREEN_SHAKE_CNT: i32 = 0;
static mut SCREEN_SHAKE_INTENSE: f32 = 0.0;

#[no_mangle]
pub extern "C" fn screen_shake_set(cnt: i32, intense: f32) {
    unsafe {
        SCREEN_SHAKE_CNT = cnt;
        SCREEN_SHAKE_INTENSE = intense;
    }
}

#[no_mangle]
pub extern "C" fn screen_shake_update() {
    unsafe {
        if SCREEN_SHAKE_CNT > 0 {
            SCREEN_SHAKE_CNT -= 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn screen_shake_apply() {
    unsafe {
        let mut shake = Vector2 { x: 0.0, y: 0.0 };

        if SCREEN_SHAKE_CNT > 0 {
            let magnitude = SCREEN_SHAKE_INTENSE * (SCREEN_SHAKE_CNT + 10) as f32;
            shake = Vector2 {
                x: rand_next_signed_float(magnitude),
                y: rand_next_signed_float(magnitude),
            }
        }

        glTranslatef(shake.x, shake.y, -20.0);
    }
}
