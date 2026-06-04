use crate::actors::actor_export::{fragments_draw_luminous, particles_draw_luminous};
use crate::rendering::gl::{glPopMatrix, glPushMatrix};
use crate::screen::{screen_end_render_to_texture, screen_start_render_to_texture};
use crate::screen_shake::screen_shake_apply;

// Game states (must match the state enum in P47GameManager.d).
pub const STATE_TITLE: i32 = 0;
pub const STATE_IN_GAME: i32 = 1;
pub const STATE_GAMEOVER: i32 = 2;
pub const STATE_PAUSE: i32 = 3;

fn in_game_draw_luminous() {
    particles_draw_luminous();
    fragments_draw_luminous();
}

#[no_mangle]
pub extern "C" fn game_manager_draw_luminous(state: i32) {
    screen_start_render_to_texture();
    unsafe { glPushMatrix() };
    screen_shake_apply();
    match state {
        STATE_IN_GAME | STATE_PAUSE | STATE_GAMEOVER => in_game_draw_luminous(),
        _ => {}
    }
    unsafe { glPopMatrix() };
    screen_end_render_to_texture();
}
