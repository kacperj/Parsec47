use crate::barrage::{
    CATEGORY_LARGEMOVE, CATEGORY_MIDDLEMOVE, CATEGORY_SMALLMOVE, CATEGORY_SMALLSIDEMOVE,
};
use crate::core::rand::{rand_next_float, rand_next_int, rand_next_signed_float};
use crate::core::vector::Vector2;
use crate::enemy::{enemies_add, enemies_add_boss};
use crate::enemy_type::{
    enemy_type_create_large, enemy_type_create_large_boss, enemy_type_create_middle,
    enemy_type_create_middle_boss, enemy_type_create_small, EnemyType, TYPE_MIDDLE, TYPE_SMALL,
};
use crate::enemy_type_tracker;
use crate::field::{
    field_get_collision_box, field_set_aim_speed, field_set_aim_z, field_set_type,
};
use crate::game_manager::STATE_TITLE;
use crate::sound::{sound_manager_fade_music, sound_manager_play_bgm};
use core::ffi::{c_float, c_int};
use std::f32::consts::PI;

const MIDDLE_RUSH_SECTION_PATTERN: c_int = 6;

// [#smalltype, #middletype, #largetype] per section pattern, indexed by mode.
const APPEARANCE_PATTERN: [[[c_int; 3]; 7]; 2] = [
    // ROLL
    [
        [1, 0, 0],
        [2, 0, 0],
        [1, 1, 0],
        [1, 0, 1],
        [2, 1, 0],
        [2, 0, 1],
        [0, 1, 1],
    ],
    // LOCK
    [
        [1, 0, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 0, 1],
        [2, 1, 0],
        [1, 1, 1],
        [0, 1, 1],
    ],
];

// Appearance point — must match the anonymous enum in StageManager.d.
const POINT_TOP: i32 = 0;
const POINT_SIDE: i32 = 1;
const POINT_BACK: i32 = 2;

// Appearance pattern — must match the anonymous enum in StageManager.d.
const PATTERN_ONE_SIDE: i32 = 0;
const PATTERN_ALTERNATE: i32 = 1;
const PATTERN_BOTH_SIDES: i32 = 2;

// Appearance position is fixed or not — must match the anonymous enum in StageManager.d.
const SEQUENCE_RANDOM: i32 = 0;
const SEQUENCE_FIXED: i32 = 1;

// Enemy kind — must match the anonymous enum in StageManager.d.
const KIND_SMALL: c_int = 0;
const KIND_MIDDLE: c_int = 1;
const KIND_LARGE: c_int = 2;

const SMALL_ENEMY_TYPE_MAX: usize = 3;
const MIDDLE_ENEMY_TYPE_MAX: usize = 4;
const LARGE_ENEMY_TYPE_MAX: usize = 2;

const FIELD_SPACE: f32 = 0.5;
const SIMULTANEOUS_APPEARANCE_MAX: usize = 4;
const FIELD_TYPE_NUM: c_int = 4; // = Field.TYPE_NUM in Field.d
const BGM_NUM: c_int = 4; // = SoundManager.BGM_NUM in SoundManager.d

/// Enemy appearance schedule for one slot. Now Rust-internal (the whole
/// StageManager state machine lives here); the D wrapper no longer sees it.
#[derive(Copy, Clone)]
pub struct EnemyAppearance {
    pub kind: EnemyType, // D field name: type
    pub point: i32,
    pub pattern: i32,
    pub sequence: i32,
    pub pos: f32,
    pub num: i32,
    pub interval: i32,
    pub group_interval: i32,
    pub cnt: i32,
    pub left: i32,
    pub side: i32,
    pub move_type: i32,
    pub move_type_random: i32,
}

impl EnemyAppearance {
    fn empty() -> Self {
        EnemyAppearance {
            kind: EnemyType::new(0),
            point: 0,
            pattern: 0,
            sequence: 0,
            pos: 0.0,
            num: 0,
            interval: 0,
            group_interval: 0,
            cnt: 0,
            left: 0,
            side: 0,
            move_type: 0,
            move_type_random: 0,
        }
    }
}

/// Spawn position + facing produced by `process_appearance`.
struct SpawnPoint {
    pos: Vector2,
    dir: f32,
}

struct StageEnemyTypes {
    small: [EnemyType; SMALL_ENEMY_TYPE_MAX],
    middle: [EnemyType; MIDDLE_ENEMY_TYPE_MAX],
    large: [EnemyType; LARGE_ENEMY_TYPE_MAX],
    middle_boss: EnemyType,
    large_boss: EnemyType,
}

static mut ENEMY_TYPES: Option<StageEnemyTypes> = None;

fn get_appearance_for_section(section: c_int, middle_rush_section_num: c_int) -> c_int {
    if section == 0 {
        return 0;
    }
    if section == middle_rush_section_num {
        return MIDDLE_RUSH_SECTION_PATTERN;
    }
    let sp = section * 3 / 7 + 1;
    let ep = 3 + section * 3 / 10;
    sp + rand_next_int(ep - sp + 1)
}

fn stage_get_appearance_count_for_section(
    section: c_int,
    middle_rush_section_num: c_int,
    mode: c_int,
    enemy_type: c_int,
) -> c_int {
    let ap = get_appearance_for_section(section, middle_rush_section_num);
    APPEARANCE_PATTERN[mode as usize][ap as usize][enemy_type as usize]
}

fn stage_create_enemy_data(rank: c_float, mode: c_int) {
    let mut type_id = 0;
    let mut next_id = || {
        let id = type_id;
        type_id += 1;
        id
    };
    let types = StageEnemyTypes {
        small: core::array::from_fn(|_| enemy_type_create_small(next_id(), rank, mode)),
        middle: core::array::from_fn(|_| enemy_type_create_middle(next_id(), rank, mode)),
        large: core::array::from_fn(|_| enemy_type_create_large(next_id(), rank, mode)),
        middle_boss: enemy_type_create_middle_boss(next_id(), rank, mode),
        large_boss: enemy_type_create_large_boss(next_id(), rank, mode),
    };
    unsafe {
        ENEMY_TYPES = Some(types);
    }
}

fn enemy_types() -> &'static StageEnemyTypes {
    unsafe {
        ENEMY_TYPES
            .as_ref()
            .expect("stage_create_enemy_data must be called first")
    }
}

fn set_appearance_pattern(ap: &mut EnemyAppearance) {
    ap.pattern = match rand_next_int(5) {
        0 => PATTERN_ONE_SIDE,
        1 | 2 => PATTERN_ALTERNATE,
        _ => PATTERN_BOTH_SIDES,
    };
    ap.sequence = match rand_next_int(3) {
        0 => SEQUENCE_RANDOM,
        _ => SEQUENCE_FIXED,
    };
}

fn set_small_appearance(ap: &mut EnemyAppearance) {
    let types = enemy_types();
    ap.kind = types.small[rand_next_int(SMALL_ENEMY_TYPE_MAX as i32) as usize];
    // f64 comparison matches D, where `nextFloat(1) > 0.2` promotes to double.
    if rand_next_float(1.0) as f64 > 0.2 {
        ap.point = POINT_TOP;
        ap.move_type = CATEGORY_SMALLMOVE;
    } else {
        ap.point = POINT_SIDE;
        ap.move_type = CATEGORY_SMALLSIDEMOVE;
    }
    set_appearance_pattern(ap);
    if ap.pattern == PATTERN_ONE_SIDE {
        ap.pattern = PATTERN_ALTERNATE;
    }
    match rand_next_int(4) {
        0 => {
            ap.num = 7 + rand_next_int(5);
            ap.group_interval = 72 + rand_next_int(15);
            ap.interval = 15 + rand_next_int(5);
        }
        1 => {
            ap.num = 5 + rand_next_int(3);
            ap.group_interval = 56 + rand_next_int(10);
            ap.interval = 20 + rand_next_int(5);
        }
        _ => {
            ap.num = 2 + rand_next_int(2);
            ap.group_interval = 45 + rand_next_int(20);
            ap.interval = 25 + rand_next_int(5);
        }
    }
}

fn set_middle_appearance(ap: &mut EnemyAppearance) {
    let types = enemy_types();
    ap.kind = types.middle[rand_next_int(MIDDLE_ENEMY_TYPE_MAX as i32) as usize];
    ap.point = POINT_TOP;
    ap.move_type = CATEGORY_MIDDLEMOVE;
    set_appearance_pattern(ap);
    match rand_next_int(3) {
        0 => {
            ap.num = 4;
            ap.group_interval = 240 + rand_next_int(150);
            ap.interval = 80 + rand_next_int(30);
        }
        1 => {
            ap.num = 2;
            ap.group_interval = 180 + rand_next_int(60);
            ap.interval = 180 + rand_next_int(20);
        }
        _ => {
            ap.num = 1;
            ap.group_interval = 150 + rand_next_int(50);
            ap.interval = 100;
        }
    }
}

fn set_large_appearance(ap: &mut EnemyAppearance) {
    let types = enemy_types();
    ap.kind = types.large[rand_next_int(LARGE_ENEMY_TYPE_MAX as i32) as usize];
    ap.point = POINT_TOP;
    ap.move_type = CATEGORY_LARGEMOVE;
    set_appearance_pattern(ap);
    match rand_next_int(3) {
        0 => {
            ap.num = 3;
            ap.group_interval = 400 + rand_next_int(100);
            ap.interval = 240 + rand_next_int(40);
        }
        1 => {
            ap.num = 2;
            ap.group_interval = 400 + rand_next_int(60);
            ap.interval = 300 + rand_next_int(20);
        }
        _ => {
            ap.num = 1;
            ap.group_interval = 270 + rand_next_int(50);
            ap.interval = 200;
        }
    }
}

fn stage_create_appearance(kind: c_int) -> EnemyAppearance {
    let mut ap = EnemyAppearance::empty();
    match kind {
        KIND_SMALL => set_small_appearance(&mut ap),
        KIND_MIDDLE => set_middle_appearance(&mut ap),
        KIND_LARGE => set_large_appearance(&mut ap),
        _ => {}
    }
    ap.cnt = 0;
    ap.left = ap.num;
    ap.side = rand_next_int(2) * 2 - 1;
    ap.pos = rand_next_float(1.0);
    // D used to roll this separately right after createAppearance; folded in
    // here in the same RNG-call order.
    ap.move_type_random = rand_next_int(i32::MAX);
    ap
}

// ---- Stage state machine (port of the StageManager class in StageManager.d) ----

fn field_half() -> (f32, f32) {
    let b = field_get_collision_box();
    ((b.x2 - b.x1) * 0.5, (b.y2 - b.y1) * 0.5)
}

/// Port of StageManager.processAppearance: pick the spawn position/facing for the
/// next enemy of this schedule slot, and advance the slot's group/side counters.
fn process_appearance(ap: &mut EnemyAppearance) -> SpawnPoint {
    let (hw, hh) = field_half();
    // RANDOM rolls a fresh position; FIXED reuses ap.pos (no RNG call) — same
    // call order as the D switch.
    let p = match ap.sequence {
        SEQUENCE_FIXED => ap.pos,
        _ => rand_next_float(1.0),
    };
    let mut apos = Vector2 { x: 0.0, y: 0.0 };
    let mut d = 0.0;
    match ap.point {
        POINT_TOP => {
            apos.x = if ap.pattern == PATTERN_BOTH_SIDES {
                (p - 0.5) * hw * 1.8
            } else {
                (p * 0.6 + 0.2) * hw * ap.side as f32
            };
            apos.y = hh - FIELD_SPACE;
            d = PI;
        }
        POINT_BACK => {
            apos.x = if ap.pattern == PATTERN_BOTH_SIDES {
                (p - 0.5) * hw * 1.8
            } else {
                (p * 0.6 + 0.2) * hw * ap.side as f32
            };
            apos.y = -hh + FIELD_SPACE;
            d = 0.0;
        }
        POINT_SIDE => {
            apos.x = if ap.pattern == PATTERN_BOTH_SIDES {
                (hw - FIELD_SPACE) * (rand_next_int(2) * 2 - 1) as f32
            } else {
                (hw - FIELD_SPACE) * ap.side as f32
            };
            apos.y = (p * 0.4 + 0.4) * hh;
            d = if apos.x < 0.0 { PI / 2.0 } else { PI / 2.0 * 3.0 };
        }
        _ => {}
    }
    apos.x *= 0.88;
    ap.left -= 1;
    if ap.left <= 0 {
        ap.cnt = ap.group_interval;
        ap.left = ap.num;
        if ap.pattern != PATTERN_ONE_SIDE {
            ap.side *= -1;
        }
        ap.pos = rand_next_float(1.0);
    } else {
        ap.cnt = ap.interval;
    }
    SpawnPoint { pos: apos, dir: d }
}

/// Manage the stage data (enemies' appearance). Port of StageManager.d's class
/// state and its move/section logic. `mode`/`state` are passed in from the game
/// manager (matching how enemies_move takes `mode`).
struct StageManager {
    parsec: i32,
    boss_section: bool,
    middle_boss_type: EnemyType,
    large_boss_type: EnemyType,
    appearance: [EnemyAppearance; SIMULTANEOUS_APPEARANCE_MAX],
    ap_num: i32,
    section_cnt: i32,
    section_interval_cnt: i32,
    section: i32,
    rank: f32,
    rank_inc: f32,
    middle_rush_section_num: i32,
    middle_rush_section: bool,
    stage_type: i32,
}

impl StageManager {
    fn new() -> Self {
        StageManager {
            parsec: 0,
            boss_section: false,
            middle_boss_type: EnemyType::new(0),
            large_boss_type: EnemyType::new(0),
            appearance: [EnemyAppearance::empty(); SIMULTANEOUS_APPEARANCE_MAX],
            ap_num: 0,
            section_cnt: 0,
            section_interval_cnt: 0,
            section: 0,
            rank: 0.0,
            rank_inc: 0.0,
            middle_rush_section_num: 0,
            middle_rush_section: false,
            stage_type: 0,
        }
    }

    fn create_enemy_data(&mut self, mode: c_int) {
        stage_create_enemy_data(self.rank, mode);
        let types = enemy_types();
        self.middle_boss_type = types.middle_boss;
        self.large_boss_type = types.large_boss;
    }

    fn create_section_data(&mut self, mode: c_int) {
        self.ap_num = 0;
        if self.rank <= 0.0 {
            return;
        }
        field_set_aim_speed(0.1 + self.section as f32 * 0.02);
        if self.section == 4 {
            // Set the middle boss.
            let (_, hh) = field_half();
            let py = hh / 4.0 * 3.0;
            enemies_add_boss(0.0, py, PI, &self.middle_boss_type as *const EnemyType);
            self.boss_section = true;
            self.section_interval_cnt = 2 * 60;
            self.section_cnt = 2 * 60;
            field_set_aim_z(11.0);
            return;
        } else if self.section == 9 {
            // Set the large boss.
            let (_, hh) = field_half();
            let py = hh / 4.0 * 3.0;
            enemies_add_boss(0.0, py, PI, &self.large_boss_type as *const EnemyType);
            self.boss_section = true;
            self.section_interval_cnt = 3 * 60;
            self.section_cnt = 3 * 60;
            field_set_aim_z(12.0);
            return;
        } else if self.section == self.middle_rush_section_num {
            // In this section, no small enemy.
            self.middle_rush_section = true;
            field_set_aim_z(9.0);
        } else {
            self.middle_rush_section = false;
            field_set_aim_z(10.0 + rand_next_signed_float(0.3));
        }
        self.boss_section = false;
        // D had a duplicate `else if (section == 3)` dead branch; both arms set
        // 1 minute, so it collapses to this.
        if self.section == 3 {
            self.section_interval_cnt = 2 * 60;
        } else {
            self.section_interval_cnt = 1 * 60;
        }
        self.section_cnt = self.section_interval_cnt + 10 * 60;

        for enemy_type in [KIND_SMALL, KIND_MIDDLE, KIND_LARGE] {
            let number_of_enemy_type = stage_get_appearance_count_for_section(
                self.section,
                self.middle_rush_section_num,
                mode,
                enemy_type,
            );
            for _ in 0..number_of_enemy_type {
                self.appearance[self.ap_num as usize] = stage_create_appearance(enemy_type);
                self.ap_num += 1;
            }
        }
    }

    fn create_stage(&mut self, mode: c_int) {
        self.create_enemy_data(mode);
        self.middle_rush_section_num = 2 + rand_next_int(6);
        if self.middle_rush_section_num <= 4 {
            self.middle_rush_section_num += 1;
        }
        field_set_type(self.stage_type % FIELD_TYPE_NUM);
        sound_manager_play_bgm(self.stage_type % BGM_NUM);
        self.stage_type += 1;
    }

    fn goto_next_section(&mut self, mode: c_int, state: c_int) {
        self.section += 1;
        self.parsec += 1;
        if state == STATE_TITLE && self.section >= 4 {
            self.section = 0;
            self.parsec -= 4;
        }
        if self.section >= 10 {
            self.section = 0;
            self.rank += self.rank_inc;
            self.create_stage(mode);
        }
        self.create_section_data(mode);
    }

    fn set_rank(
        &mut self,
        base_rank: f32,
        inc: f32,
        start_parsec: i32,
        type_: i32,
        mode: c_int,
        state: c_int,
    ) {
        self.rank = base_rank;
        self.rank_inc = inc;
        // start_parsec / 10 is integer division in D (both operands int).
        self.rank += self.rank_inc * (start_parsec / 10) as f32;
        self.section = -1;
        self.parsec = start_parsec - 1;
        self.stage_type = type_;
        self.create_stage(mode);
        self.goto_next_section(mode, state);
    }

    fn do_move(&mut self, mode: c_int, state: c_int) {
        for i in 0..self.ap_num as usize {
            self.appearance[i].cnt -= 1;
            if self.appearance[i].cnt > 0 {
                // Force the extra enemy out early so every type is seen at least once.
                let kind = self.appearance[i].kind.kind;
                let id = self.appearance[i].kind.id;
                if !self.middle_rush_section {
                    if kind == TYPE_SMALL && !enemy_type_tracker::exists(id) {
                        self.appearance[i].cnt = 0;
                        enemy_type_tracker::mark(id);
                    }
                } else if kind == TYPE_MIDDLE && !enemy_type_tracker::exists(id) {
                    self.appearance[i].cnt = 0;
                    enemy_type_tracker::mark(id);
                }
                continue;
            }
            let sp = process_appearance(&mut self.appearance[i]);
            let kind = self.appearance[i].kind;
            let move_type = self.appearance[i].move_type;
            let move_type_random = self.appearance[i].move_type_random;
            enemies_add(
                sp.pos.x,
                sp.pos.y,
                sp.dir,
                &kind as *const EnemyType,
                move_type,
                move_type_random,
            );
        }

        if !self.boss_section
            || (!enemy_type_tracker::exists(self.middle_boss_type.id)
                && !enemy_type_tracker::exists(self.large_boss_type.id))
        {
            self.section_cnt -= 1;
        }

        if self.section_cnt < self.section_interval_cnt {
            if self.section == 9 && self.section_cnt == self.section_interval_cnt - 1 {
                sound_manager_fade_music();
            }
            self.ap_num = 0;
            if self.section_cnt <= 0 {
                self.goto_next_section(mode, state);
            }
        }

        enemy_type_tracker::clear();
    }
}

static mut STAGE: Option<StageManager> = None;

fn stage() -> &'static mut StageManager {
    unsafe { STAGE.get_or_insert_with(StageManager::new) }
}

pub fn stage_manager_init() {
    unsafe {
        STAGE = Some(StageManager::new());
    }
}

pub fn stage_manager_set_rank(
    base_rank: c_float,
    inc: c_float,
    start_parsec: c_int,
    type_: c_int,
    mode: c_int,
    state: c_int,
) {
    stage().set_rank(base_rank, inc, start_parsec, type_, mode, state);
}

pub fn stage_manager_move(mode: c_int, state: c_int) {
    stage().do_move(mode, state);
}

pub fn stage_manager_get_parsec() -> c_int {
    stage().parsec
}

pub fn stage_manager_is_boss_section() -> bool {
    stage().boss_section
}
