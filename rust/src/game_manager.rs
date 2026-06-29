use crate::actors::actor_export::{
    fragments_draw, fragments_draw_luminous, locks_draw, particles_draw, particles_draw_luminous,
    rolls_draw, shots_draw,
};
use crate::actors::bonus::bonuses_draw;
use crate::bullets::bullet_actor_pool::bullets_draw;
use crate::enemy::enemies_draw;
use crate::field::field_draw;
use crate::rendering::gl::{glPopMatrix, glPushMatrix};
use crate::screen::{
    screen_clear, screen_draw_luminous, screen_end_render_to_texture,
    screen_start_render_to_texture, screen_view_ortho_fixed, screen_view_perspective,
};
use crate::screen_shake::screen_shake_apply;
use crate::ship::ship_draw;
use crate::stage_manager::{stage_manager_get_parsec, stage_manager_is_boss_section};
use crate::title::title_draw;
use crate::ui_renderer::{
    renderer_draw_box, renderer_draw_gameover_status, renderer_draw_pause_status,
    renderer_draw_score, renderer_draw_side_boards, renderer_draw_side_info, PAUSE_RESUME,
    PAUSE_SURRENDER,
};

// Game-logic functions called by the state machine ported below (formerly the
// `private extern(C)` block in P47GameManager.d).
use crate::actors::actor_export::{
    fragments_update, locks_add, locks_clear, locks_release_all, locks_update, particles_update,
    rolls_clear, rolls_update, shots_clear, shots_update,
};
use crate::actors::bonus::{bonuses_clear, bonuses_init, bonuses_move, bonuses_set_speed_rate};
use crate::barrage::barrage_export::{barrage_load_bulletmls, barrage_unload_bulletmls};
use crate::bullet_actor::bullet_actor_create_display_lists;
use crate::bullets::bullet_actor_pool::{
    bullets_clear, bullets_get_total_speed, bullets_reset_total_speed, bullets_to_retro_all,
    bullets_update,
};
use crate::core::rand::rand_next_int;
use crate::enemy::{enemies_clear, enemies_move, enemies_push_lock_targets};
use crate::field::{field_create_ring_display_list, field_init, field_move, field_set_color};
use crate::letter_render::letter_render_create_display_lists;
use crate::pad::{
    is_down_pressed, is_fire_button_pressed, is_pause_pressed, is_quit_pressed,
    is_special_button_pressed, is_up_pressed, pad_take_controller_unplugged,
};
use crate::prefs::{
    prefs_get_hi_score, prefs_get_reached_parsec, prefs_get_selected_difficulty,
    prefs_get_selected_mode, prefs_get_selected_parsec_slot, prefs_get_start_parsec,
    prefs_set_hi_score, prefs_set_reached_parsec, prefs_set_selected_difficulty,
    prefs_set_selected_mode, prefs_set_selected_parsec_slot,
};
use crate::screen_shake::{screen_shake_set, screen_shake_update};
use crate::ship::{
    ship_create_display_lists, ship_move, ship_set_cnt, ship_set_speed_rate, ship_start,
};
use crate::sound::{
    sound_manager_close, sound_manager_fade_music, sound_manager_init, sound_manager_set_in_game,
    sound_manager_stop_music,
};
use crate::stage_manager::{stage_manager_init, stage_manager_move, stage_manager_set_rank};
use crate::state::state_export::score_state;
use crate::title::{
    title_change_mode, title_get_cur_x, title_get_cur_y, title_get_mode, title_move,
    title_should_change_stage, title_start,
};
use crate::ui_renderer::{renderer_title_texture_delete, renderer_title_texture_init};

// Game states (must match the state enum in P47GameManager.d).
pub const STATE_TITLE: i32 = 0;
pub const STATE_IN_GAME: i32 = 1;
pub const STATE_GAMEOVER: i32 = 2;
pub const STATE_PAUSE: i32 = 3;

// Ship mode (= ShipMode.d). Only ROLL needs distinguishing here.
const MODE_ROLL: i32 = 0;

// Frames the selected pause box takes to grow to full size (matches title BOX_COUNT).
const PAUSE_BOX_COUNT: i32 = 16;

// ── Game-manager state machine (port of the P47GameManager class) ─────────────
//
// Holds the game-flow state formerly owned by the D class: the title/in-game/
// gameover/pause state machine, difficulty/mode/parsec selection, ship & bullet
// event handling, and the adaptive frame-interval slowdown. The pools and
// managers it drives already live in Rust; here we orchestrate them.

// Difficulty levels (= the public difficulty enum in P47GameManager.d).
const PRACTICE: i32 = 0;
const NORMAL: i32 = 1;
const HARD: i32 = 2;
const EXTREME: i32 = 3;
const QUIT: i32 = 4;

const MODE_NUM: usize = 2;
const DIFFICULTY_NUM: i32 = 4;
const INTERVAL_BASE: i32 = 16;

// Event bits returned from ship_move() and bullets_update().
const EVENT_ADD_LOCK: i32 = 1;
const EVENT_RELEASE_LOCK: i32 = 2;
const EVENT_DESTROYED: i32 = 4;

// Intentional slowdown threshold, indexed by ship mode (ROLL, LOCK).
const SLOWDOWN_START_BULLETS_SPEED: [f32; MODE_NUM] = [30.0, 42.0];

#[derive(Clone, Copy)]
struct StageSelection {
    difficulty: i32,
    parsec_slot: i32,
    mode: i32,
}

struct GameManager {
    state: i32,
    stage_selection: StageSelection,
    cnt: i32,
    pause_cnt: i32,
    // Frame interval (ms) read by MainLoop each frame; modulated for slowdown.
    interval: i32,
    interval_smooth: f32,
    nowait: bool,
    no_field: bool,
    no_bonus: bool,
    p_prsd: bool,   // pause-key edge detect
    btn_prsd: bool, // action-button edge detect
    pad_prsd: bool, // direction-pad edge detect (pause menu)
    pause_cursor: i32, // selected pause-menu button
    pause_box_cnt: i32, // selected pause box grow animation
}

static mut GAME_MANAGER: Option<GameManager> = None;

fn game_manager() -> &'static mut GameManager {
    unsafe { GAME_MANAGER.get_or_insert_with(GameManager::new) }
}

impl GameManager {
    fn new() -> Self {
        GameManager {
            state: STATE_TITLE,
            stage_selection: StageSelection {
                difficulty: 0,
                parsec_slot: 0,
                mode: 0,
            },
            cnt: 0,
            pause_cnt: 0,
            interval: INTERVAL_BASE,
            interval_smooth: INTERVAL_BASE as f32,
            nowait: false,
            no_field: false,
            no_bonus: false,
            p_prsd: true,
            btn_prsd: true,
            pad_prsd: true,
            pause_cursor: PAUSE_RESUME,
            pause_box_cnt: PAUSE_BOX_COUNT,
        }
    }

    fn parsec(&self) -> i32 {
        stage_manager_get_parsec()
    }

    fn set_stage_rank(&self, base_rank: f32, inc: f32, start_parsec: i32, type_: i32) {
        stage_manager_set_rank(
            base_rank,
            inc,
            start_parsec,
            type_,
            self.stage_selection.mode,
            self.state,
        );
    }

    // Initialize actor pools, load BGMs/SEs and textures.
    fn init(&mut self) {
        self.stage_selection.difficulty = prefs_get_selected_difficulty();
        self.stage_selection.parsec_slot = prefs_get_selected_parsec_slot();
        self.stage_selection.mode = prefs_get_selected_mode();

        field_create_ring_display_list();
        field_init(11.0, 16.0);
        ship_create_display_lists();
        bullet_actor_create_display_lists();
        letter_render_create_display_lists();
        bonuses_init();
        barrage_load_bulletmls();
        stage_manager_init();
        renderer_title_texture_init();
        self.interval_smooth = INTERVAL_BASE as f32;
        self.interval = INTERVAL_BASE;
        let _ = sound_manager_init();
    }

    fn start(&mut self) {
        self.start_title();
    }

    fn close(&mut self) {
        barrage_unload_bulletmls();
        renderer_title_texture_delete();
        sound_manager_close();
    }

    fn ship_destroyed(&mut self) {
        bullets_to_retro_all();
        score_state().decrease_life();
        if score_state().get_life() < 0 {
            self.start_gameover();
        }
    }

    fn handle_ship_events(&mut self, events: i32) {
        if events & EVENT_ADD_LOCK != 0 {
            locks_add();
        }
        if events & EVENT_RELEASE_LOCK != 0 {
            locks_release_all();
        }
        if events & EVENT_DESTROYED != 0 {
            self.ship_destroyed();
        }
    }

    fn handle_bullet_events(&mut self, events: i32) {
        if events & EVENT_RELEASE_LOCK != 0 {
            locks_release_all();
        }
        if events & EVENT_DESTROYED != 0 {
            self.ship_destroyed();
        }
    }

    fn get_status(&self) -> StageSelection {
        StageSelection {
            difficulty: title_get_cur_y(),
            parsec_slot: title_get_cur_x(),
            mode: title_get_mode(),
        }
    }

    fn start_stage_preview(&mut self) {
        let sel = self.get_status();
        self.start_stage(sel);
    }

    fn start_stage(&mut self, stage_select: StageSelection) {
        let start_parsec = prefs_get_start_parsec(
            stage_select.mode,
            stage_select.difficulty,
            stage_select.parsec_slot,
        );

        enemies_clear();
        bullets_clear();

        self.stage_selection = stage_select;

        let stage_type = rand_next_int(99999);
        match self.stage_selection.difficulty {
            PRACTICE => {
                self.set_stage_rank(1.0, 4.0, start_parsec, stage_type);
                ship_set_speed_rate(0.7);
                bonuses_set_speed_rate(0.6);
            }
            NORMAL => {
                self.set_stage_rank(10.0, 8.0, start_parsec, stage_type);
                ship_set_speed_rate(0.9);
                bonuses_set_speed_rate(0.8);
            }
            HARD => {
                self.set_stage_rank(22.0, 12.0, start_parsec, stage_type);
                ship_set_speed_rate(1.0);
                bonuses_set_speed_rate(1.0);
            }
            EXTREME => {
                self.set_stage_rank(36.0, 16.0, start_parsec, stage_type);
                ship_set_speed_rate(1.2);
                bonuses_set_speed_rate(1.3);
            }
            QUIT => {
                self.set_stage_rank(0.0, 0.0, 0, 0);
                ship_set_speed_rate(1.0);
                bonuses_set_speed_rate(1.0);
            }
            _ => {}
        }
    }

    fn init_ship_state(&self) {
        score_state().set_initial();
        ship_start(self.stage_selection.mode);
    }

    fn start_in_game(&mut self) {
        self.state = STATE_IN_GAME;
        sound_manager_set_in_game((self.state == STATE_IN_GAME) as i32);
        self.init_ship_state();
        let sel = self.stage_selection;
        self.start_stage(sel);
    }

    fn start_title(&mut self) {
        self.state = STATE_TITLE;
        title_start(
            self.stage_selection.difficulty,
            self.stage_selection.parsec_slot,
            self.stage_selection.mode,
        );
        field_set_color(self.stage_selection.mode);
        self.init_ship_state();
        bullets_clear();
        bonuses_clear();
        ship_set_cnt(0);
        self.start_stage_preview();
        self.cnt = 0;
        sound_manager_stop_music();
    }

    fn start_gameover(&mut self) {
        self.state = STATE_GAMEOVER;
        bonuses_clear();
        shots_clear();
        rolls_clear();
        locks_clear();
        screen_shake_set(0, 0.0);
        self.interval_smooth = INTERVAL_BASE as f32;
        self.interval = INTERVAL_BASE;
        self.cnt = 0;
        let sel = self.stage_selection;
        if score_state().get_score() > prefs_get_hi_score(sel.mode, sel.difficulty, sel.parsec_slot) {
            prefs_set_hi_score(sel.mode, sel.difficulty, sel.parsec_slot, score_state().get_score());
        }
        if self.parsec() > prefs_get_reached_parsec(sel.mode, sel.difficulty) {
            prefs_set_reached_parsec(sel.mode, sel.difficulty, self.parsec());
        }
        sound_manager_fade_music();
    }

    fn start_pause(&mut self) {
        self.state = STATE_PAUSE;
        self.pause_cnt = 0;
        self.pause_cursor = PAUSE_RESUME;
        self.pause_box_cnt = PAUSE_BOX_COUNT;
        self.btn_prsd = true;
        self.pad_prsd = true;
    }

    fn resume_pause(&mut self) {
        self.state = STATE_IN_GAME;
    }

    fn in_game_move(&mut self) {
        stage_manager_move(self.stage_selection.mode, self.state);
        field_move();
        let events = ship_move();
        self.handle_ship_events(events);
        bonuses_move();
        shots_update();
        enemies_move(self.stage_selection.mode);
        if self.stage_selection.mode == MODE_ROLL {
            rolls_update();
        } else {
            enemies_push_lock_targets();
            locks_update();
        }
        bullets_reset_total_speed();
        let bullet_events = bullets_update();
        self.handle_bullet_events(bullet_events);
        particles_update();
        fragments_update();
        screen_shake_update();
        // Losing the controller mid-game opens the pause menu so play can't
        // continue with input silently gone. Latched in the SDL event handler;
        // `pad_take_controller_unplugged` clears it so this fires once per unplug.
        if pad_take_controller_unplugged() {
            self.p_prsd = true;
            self.start_pause();
            return;
        }
        // Pause on the pause key/Start button, or when quit (Escape/Back) is
        // pressed during play — quitting from gameplay opens the pause menu
        // instead of exiting the game.
        if is_pause_pressed() || is_quit_pressed() {
            if !self.p_prsd {
                self.p_prsd = true;
                self.start_pause();
            }
        } else {
            self.p_prsd = false;
        }
        if !self.nowait {
            // Intentional slowdown when the total bullet speed exceeds the
            // mode-specific threshold.
            let threshold = SLOWDOWN_START_BULLETS_SPEED[self.stage_selection.mode as usize];
            if bullets_get_total_speed() > threshold {
                let mut sm = bullets_get_total_speed() / threshold;
                if sm > 1.75 {
                    sm = 1.75;
                }
                self.interval_smooth += (sm * INTERVAL_BASE as f32 - self.interval_smooth) * 0.1;
                self.interval = self.interval_smooth as i32;
            } else {
                self.interval_smooth += (INTERVAL_BASE as f32 - self.interval_smooth) * 0.08;
                self.interval = self.interval_smooth as i32;
            }
        }
    }

    // Returns true when the player chose QUIT from the title menu.
    fn title_move(&mut self) -> bool {
        title_move();
        if title_should_change_stage() != 0 {
            self.start_stage_preview();
        }

        if self.cnt <= 8 {
            self.btn_prsd = true;
        } else if is_fire_button_pressed() {
            if !self.btn_prsd {
                self.stage_selection = self.get_status();
                if self.stage_selection.difficulty >= DIFFICULTY_NUM {
                    return true;
                } else {
                    prefs_set_selected_difficulty(self.stage_selection.difficulty);
                    prefs_set_selected_parsec_slot(self.stage_selection.parsec_slot);
                    prefs_set_selected_mode(self.stage_selection.mode);
                    self.start_in_game();
                }
                return false;
            }
        } else if is_special_button_pressed() {
            if !self.btn_prsd {
                title_change_mode();
                self.start_stage_preview();
                field_set_color(self.stage_selection.mode);
                self.btn_prsd = true;
            }
        } else {
            self.btn_prsd = false;
        }

        stage_manager_move(self.stage_selection.mode, self.state);
        field_move();
        enemies_move(self.stage_selection.mode);
        bullets_update();
        false
    }

    fn gameover_move(&mut self) {
        let mut goto_next_state = false;
        if self.cnt <= 64 {
            self.btn_prsd = true;
        } else if is_fire_button_pressed() || is_special_button_pressed() {
            if !self.btn_prsd {
                goto_next_state = true;
            }
        } else {
            self.btn_prsd = false;
        }
        if self.cnt > 64 && goto_next_state {
            self.start_title();
        } else if self.cnt > 500 {
            self.start_title();
        }
        field_move();
        enemies_move(self.stage_selection.mode);
        bullets_update();
        particles_update();
        fragments_update();
    }

    fn pause_move(&mut self) {
        self.pause_cnt += 1;

        // Pause key still toggles straight back into the game.
        if is_pause_pressed() {
            if !self.p_prsd {
                self.p_prsd = true;
                self.resume_pause();
                return;
            }
        } else {
            self.p_prsd = false;
        }

        // Move the cursor between the Resume / Surrender buttons.
        if is_up_pressed() || is_down_pressed() {
            if !self.pad_prsd {
                self.pad_prsd = true;
                self.pause_cursor = if self.pause_cursor == PAUSE_RESUME {
                    PAUSE_SURRENDER
                } else {
                    PAUSE_RESUME
                };
                self.pause_box_cnt = PAUSE_BOX_COUNT;
            }
        } else {
            self.pad_prsd = false;
        }

        if self.pause_box_cnt >= 0 {
            self.pause_box_cnt -= 1;
        }

        // Activate the selected button.
        if is_fire_button_pressed() {
            if !self.btn_prsd {
                self.btn_prsd = true;
                match self.pause_cursor {
                    PAUSE_SURRENDER => self.start_gameover(),
                    _ => self.resume_pause(),
                }
            }
        } else {
            self.btn_prsd = false;
        }
    }

    // Returns nonzero when the game should quit.
    fn move_(&mut self) -> i32 {
        sound_manager_set_in_game((self.state == STATE_IN_GAME) as i32);
        match self.state {
            STATE_IN_GAME => self.in_game_move(),
            STATE_TITLE => {
                if self.title_move() {
                    return 1;
                }
            }
            STATE_GAMEOVER => self.gameover_move(),
            STATE_PAUSE => self.pause_move(),
            _ => {}
        }
        self.cnt += 1;
        0
    }
}

// Boss shield meter, fed by the enemy pool (enemy.rs) each frame and read by
// P47GameManager.drawBossShieldMeter (still D). Port of P47GameManager's
// setBossShieldMeter + bossShield/bossWingShield fields.
const BOSS_WING_NUM: usize = 4;
static mut BOSS_SHIELD: i32 = 0;
static mut BOSS_WING_SHIELD: [i32; BOSS_WING_NUM] = [0; BOSS_WING_NUM];

#[allow(clippy::too_many_arguments)]
pub fn set_boss_shield_meter(bs: i32, s1: i32, s2: i32, s3: i32, s4: i32, r: f32) {
    let r = r * 0.7;
    unsafe {
        BOSS_SHIELD = (bs as f32 * r) as i32;
        BOSS_WING_SHIELD[0] = (s1 as f32 * r) as i32;
        BOSS_WING_SHIELD[1] = (s2 as f32 * r) as i32;
        BOSS_WING_SHIELD[2] = (s3 as f32 * r) as i32;
        BOSS_WING_SHIELD[3] = (s4 as f32 * r) as i32;
    }
}

pub fn game_manager_get_boss_shield() -> i32 {
    unsafe { BOSS_SHIELD }
}

pub fn game_manager_get_boss_wing_shield(i: i32) -> i32 {
    unsafe { BOSS_WING_SHIELD[i as usize] }
}

fn in_game_draw_luminous() {
    particles_draw_luminous();
    fragments_draw_luminous();
}

pub fn game_manager_draw_luminous(state: i32) {
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

// Scene drawing (port of P47GameManager.inGameDraw/titleDraw/gameoverDraw).
// `no_field`/`no_bonus` mirror Field.noField / P47GameManager.noBonus (boot
// flags still owned by D); `mode` is the current ShipMode.

pub fn game_manager_in_game_draw(no_field: bool, no_bonus: bool, mode: i32) {
    if !no_field {
        field_draw();
    }
    if !no_bonus {
        bonuses_draw();
    }
    particles_draw();
    fragments_draw();
    ship_draw();
    shots_draw();
    if mode == MODE_ROLL {
        rolls_draw();
    } else {
        locks_draw();
    }
    enemies_draw();
    bullets_draw();
}

pub fn game_manager_title_draw(no_field: bool) {
    if !no_field {
        field_draw();
    }
    enemies_draw();
    bullets_draw();
}

pub fn game_manager_gameover_draw(no_field: bool) {
    if !no_field {
        field_draw();
    }
    particles_draw();
    fragments_draw();
    enemies_draw();
    bullets_draw();
}

// Status overlay (port of P47GameManager.titleDrawStatus).
pub fn game_manager_title_draw_status() {
    renderer_draw_side_boards();
    renderer_draw_score();
    title_draw();
}

// In-game status overlay (port of P47GameManager.inGameDrawStatus).
fn in_game_draw_status() {
    renderer_draw_side_info(stage_manager_get_parsec());
    if stage_manager_is_boss_section() {
        game_manager_draw_boss_shield_meter();
    }
}

// Full per-frame draw (port of P47GameManager.draw). The six values are pulled
// from the GameManager state by `game_manager_draw` below.
fn draw_frame(state: i32, mode: i32, cnt: i32, pause_cnt: i32, pause_cursor: i32, pause_box_cnt: i32, no_field: bool, no_bonus: bool) {
    game_manager_draw_luminous(state);

    screen_clear();
    unsafe { glPushMatrix() };
    screen_shake_apply();
    match state {
        STATE_IN_GAME | STATE_PAUSE => game_manager_in_game_draw(no_field, no_bonus, mode),
        STATE_TITLE => game_manager_title_draw(no_field),
        STATE_GAMEOVER => game_manager_gameover_draw(no_field),
        _ => {}
    }
    unsafe { glPopMatrix() };

    screen_draw_luminous();

    screen_view_ortho_fixed();
    match state {
        STATE_IN_GAME => in_game_draw_status(),
        STATE_TITLE => game_manager_title_draw_status(),
        STATE_GAMEOVER => renderer_draw_gameover_status(stage_manager_get_parsec(), cnt),
        STATE_PAUSE => {
            renderer_draw_pause_status(stage_manager_get_parsec(), pause_cnt, pause_cursor, pause_box_cnt)
        }
        _ => {}
    }
    screen_view_perspective();
}

// ── Public game-manager API (formerly the public P47GameManager methods) ──────

pub fn game_manager_init() {
    game_manager().init();
}

pub fn game_manager_start() {
    game_manager().start();
}

pub fn game_manager_close() {
    game_manager().close();
}

// Returns nonzero when the game should quit (D move() returned bool).
pub fn game_manager_move() -> i32 {
    game_manager().move_()
}

// Frame interval (ms) read by MainLoop each frame.
pub fn game_manager_get_interval() -> i32 {
    game_manager().interval
}

pub fn game_manager_draw() {
    let gm = game_manager();
    draw_frame(
        gm.state,
        gm.stage_selection.mode,
        gm.cnt,
        gm.pause_cnt,
        gm.pause_cursor,
        gm.pause_box_cnt,
        gm.no_field,
        gm.no_bonus,
    );
}

pub fn game_manager_set_nowait(v: i32) {
    game_manager().nowait = v != 0;
}

pub fn game_manager_set_no_field(v: i32) {
    game_manager().no_field = v != 0;
}

pub fn game_manager_set_no_bonus(v: i32) {
    game_manager().no_bonus = v != 0;
}

// Boss shield/wing meters (port of P47GameManager.drawBossShieldMeter).
pub fn game_manager_draw_boss_shield_meter() {
    renderer_draw_box(165, 6, game_manager_get_boss_shield(), 6);
    let mut y = 24;
    for i in 0..BOSS_WING_NUM {
        let wing_shield = game_manager_get_boss_wing_shield(i as i32);
        if i % 2 == 0 {
            renderer_draw_box(165, y, wing_shield, 6);
        } else {
            renderer_draw_box(475 - wing_shield, y, wing_shield, 6);
            y += 12;
        }
    }
}
