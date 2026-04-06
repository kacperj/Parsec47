use crate::renderer::*;
use crate::rendering::color::*;
use core::ffi::{c_float, c_int};

const RETRO_CNT: i32 = 20;
const BOX_SIZE: f32 = 0.4;
const BONUS_COLOR: Color = Color {
    r: 0.2,
    g: 0.7,
    b: 0.5,
    a: 1.0,
};

#[no_mangle]
pub extern "C" fn bonus_draw(
    pos_x: c_float,
    pos_y: c_float,
    cnt: c_int,
    is_down: bool,
    is_inhaled: bool,
) {
    let retro = if cnt < RETRO_CNT {
        1.0 - cnt as f32 / RETRO_CNT as f32
    } else {
        0.0
    };
    let d = cnt as f32 * 0.1;
    let ox = d.sin() * 0.3;
    let oy = d.cos() * 0.3;

    if retro > 0.0 {
        draw_box_retro(pos_x - ox, pos_y - oy, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
        draw_box_retro(pos_x + ox, pos_y + oy, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
        draw_box_retro(pos_x - oy, pos_y + ox, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
        draw_box_retro(pos_x + oy, pos_y - ox, BOX_SIZE / 2.0, BOX_SIZE / 2.0, 0.0, BONUS_COLOR, retro, 0.2);
    } else {
        let color = if is_inhaled {
            Color { r: 0.8, g: 0.6, b: 0.4, a: 0.7 }
        } else if is_down {
            Color { r: 0.4, g: 0.9, b: 0.6, a: 0.7 }
        } else {
            Color { r: 0.8, g: 0.9, b: 0.5, a: 0.7 }
        };
        set_color(color);
        draw_box_line(pos_x - ox - BOX_SIZE / 2.0, pos_y - oy - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
        draw_box_line(pos_x + ox - BOX_SIZE / 2.0, pos_y + oy - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
        draw_box_line(pos_x - oy - BOX_SIZE / 2.0, pos_y + ox - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
        draw_box_line(pos_x + oy - BOX_SIZE / 2.0, pos_y - ox - BOX_SIZE / 2.0, BOX_SIZE, BOX_SIZE);
    }
}
