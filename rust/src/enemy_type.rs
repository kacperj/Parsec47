use core::ffi::{c_float, c_int};
use std::f32::consts::PI;

use crate::barrage::barrage_export::barrage_create;
use crate::barrage::{
    set_barrage_rank, set_barrage_rank_slow, set_barrage_shape, Barrage, CATEGORY_LARGE,
    CATEGORY_MIDDLE, CATEGORY_MIDDLESUB, CATEGORY_MIDDLESUB_LOCK, CATEGORY_MORPH, CATEGORY_SMALL,
    CATEGORY_SMALL_LOCK, INTENSE_MORPHWEAK, INTENSE_NORMAL, INTENSE_VERYWEAK, INTENSE_WEAK,
    MODE_ROLL,
};
use crate::core::rand::{rand_next_float, rand_next_int, rand_next_signed_float};
use crate::core::vector::Vector2;
use crate::renderer::create_enemy_color;
use crate::rendering::color::Color;

pub const WING_SHAPE_POINT_NUM: usize = 3;
pub const WING_BATTERY_MAX: usize = 3;
pub const BARRAGE_PATTERN_MAX: usize = 8;
pub const BODY_SHAPE_POINT_NUM: usize = 4;
pub const BATTERY_MAX: usize = 4;

// Enemy type kinds (formerly the anonymous enum in EnemyType.d).
pub const TYPE_SMALL: i32 = 0;
pub const TYPE_MIDDLE: i32 = 1;
pub const TYPE_LARGE: i32 = 2;
pub const TYPE_MIDDLEBOSS: i32 = 3;
pub const TYPE_LARGEBOSS: i32 = 4;

const VEC_ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

/// Enemy wing with batteries. Canonical definition (formerly mirrored in
/// EnemyType.d); built and consumed entirely within Rust.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BatteryType {
    pub wing_shape_pos: [Vector2; WING_SHAPE_POINT_NUM],
    pub collision_pos: Vector2,
    pub collision_size: Vector2,
    pub battery_pos: [Vector2; WING_BATTERY_MAX],
    pub battery_num: i32,
    pub b: f32,
    pub g: f32,
    pub r: f32,
    pub barrage: [Barrage; BARRAGE_PATTERN_MAX],
    pub x_reverse_alternate: bool,
    pub shield: i32,
}

impl BatteryType {
    pub const fn empty() -> Self {
        BatteryType {
            wing_shape_pos: [VEC_ZERO; WING_SHAPE_POINT_NUM],
            collision_pos: VEC_ZERO,
            collision_size: VEC_ZERO,
            battery_pos: [VEC_ZERO; WING_BATTERY_MAX],
            battery_num: 0,
            b: 0.0,
            g: 0.0,
            r: 0.0,
            barrage: [Barrage::empty(); BARRAGE_PATTERN_MAX],
            x_reverse_alternate: false,
            shield: 0,
        }
    }
}

// Mirrors EnemyType.setBattery (D): builds the barrage pattern for a wing pair
// and spreads the battery positions along the wing edge. Takes both wings by
// value and returns the modified pair.
#[allow(clippy::too_many_arguments)]
fn set_battery_pair(
    mut bt: BatteryType,
    mut bt2: BatteryType,
    rank: f32,
    n: i32,
    barrage_type: i32,
    barrage_intense: i32,
    ptn_idx: i32,
    slow: f32,
    mode: i32,
) -> (BatteryType, BatteryType) {
    let mut br = barrage_create(barrage_type, mode);
    br = set_barrage_rank_slow(br, rank / n as f32, barrage_intense, mode, slow);
    br = set_barrage_shape(br, 0.8);
    br.x_reverse = (rand_next_int(2) * 2 - 1) as f32;
    bt.barrage[ptn_idx as usize] = br;
    let mut br2 = br;
    br2.x_reverse = -br.x_reverse;
    bt2.barrage[ptn_idx as usize] = br2;
    let x_reverse_alternate = rand_next_int(4) == 0;
    bt.x_reverse_alternate = x_reverse_alternate;
    bt2.x_reverse_alternate = x_reverse_alternate;
    let mut px = bt.wing_shape_pos[1].x;
    let mut py = bt.wing_shape_pos[1].y;
    let mpx = bt.wing_shape_pos[2].x;
    let mpy = bt.wing_shape_pos[2].y;
    for i in 0..n as usize {
        bt.battery_pos[i].x = px;
        bt.battery_pos[i].y = py;
        bt2.battery_pos[i].x = -px;
        bt2.battery_pos[i].y = py;
        px += (mpx - px) / (n - 1) as f32;
        py += (mpy - py) / (n - 1) as f32;
    }
    bt.battery_num = n;
    bt2.battery_num = n;
    (bt, bt2)
}

// Mirrors the wing outline construction (three points per wing).
fn create_wings(px: f32, py: f32, mpx: f32, mpy: f32, wrl: i32) -> [Vector2; WING_SHAPE_POINT_NUM] {
    let wrl = wrl as f32;
    [
        Vector2 {
            x: px / 4.0 * wrl,
            y: py / 4.0,
        },
        Vector2 { x: px * wrl, y: py },
        Vector2 { x: mpx * wrl, y: mpy },
    ]
}

/// Enemy specifications.
/// Canonical definition (formerly mirrored in EnemyType.d); built by the
/// enemy_type_create_* functions and consumed entirely within Rust.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct EnemyType {
    pub barrage: [Barrage; BARRAGE_PATTERN_MAX],
    pub body_shape_pos: [Vector2; BODY_SHAPE_POINT_NUM],
    pub collision_size: Vector2,
    pub wing_collision: bool,
    pub enemy_color: Color,
    pub retro_size: f32,
    pub battery_type: [BatteryType; BATTERY_MAX],
    pub battery_num: i32,
    pub shield: i32,
    pub fire_interval: i32,
    pub fire_period: i32,
    pub barrage_pattern_num: i32,
    pub id: i32,
    pub kind: i32, // formerly `type` in EnemyType.d; one of the TYPE_* constants.
}

// [body x, body x jitter, body y, body y jitter, retro size,
//  wing x, wing x jitter, wing y range, wing length] per TYPE_*.
const ENEMY_SIZES: [[f32; 9]; 5] = [
    [0.3, 0.3, 0.3, 0.1, 0.1, 1.0, 0.4, 0.6, 0.9],
    [0.4, 0.2, 0.4, 0.1, 0.15, 2.2, 0.2, 1.6, 1.0],
    [0.6, 0.3, 0.5, 0.1, 0.2, 3.0, 0.3, 1.4, 1.2],
    [0.9, 0.3, 0.7, 0.2, 0.25, 5.0, 0.6, 3.0, 1.5],
    [1.2, 0.2, 0.9, 0.1, 0.3, 7.0, 0.8, 4.5, 1.5],
];

impl EnemyType {
    pub(crate) fn new(id: i32) -> Self {
        EnemyType {
            barrage: [Barrage::empty(); BARRAGE_PATTERN_MAX],
            body_shape_pos: [VEC_ZERO; BODY_SHAPE_POINT_NUM],
            collision_size: VEC_ZERO,
            wing_collision: false,
            enemy_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            },
            retro_size: 0.0,
            battery_type: [BatteryType::empty(); BATTERY_MAX],
            battery_num: 0,
            shield: 0,
            fire_interval: 0,
            fire_period: 0,
            barrage_pattern_num: 0,
            id,
            kind: 0,
        }
    }

    // Mirrors EnemyType.setEnemyShapeAndWings (D): randomizes the body quad,
    // collision box and wing pairs for the given size class.
    fn set_enemy_shape_and_wings(&mut self, size: i32) {
        let color_type = rand_next_int(3);
        self.enemy_color = create_enemy_color(color_type);

        let enemy_size = &ENEMY_SIZES[size as usize];

        let x1 = enemy_size[0] + rand_next_signed_float(enemy_size[1]);
        let y1 = enemy_size[2] + rand_next_signed_float(enemy_size[3]);
        let x2 = enemy_size[0] + rand_next_signed_float(enemy_size[1]);
        let y2 = enemy_size[2] + rand_next_signed_float(enemy_size[3]);

        self.body_shape_pos = [
            Vector2 { x: -x1, y: y1 },
            Vector2 { x: x1, y: y1 },
            Vector2 { x: x2, y: -y2 },
            Vector2 { x: -x2, y: -y2 },
        ];

        self.retro_size = enemy_size[4];
        match size {
            TYPE_SMALL | TYPE_MIDDLE | TYPE_MIDDLEBOSS => self.battery_num = 2,
            TYPE_LARGE | TYPE_LARGEBOSS => self.battery_num = 4,
            _ => {}
        }
        self.collision_size.x = if x1 > x2 { x1 } else { x2 };
        self.collision_size.y = if y1 > y2 { y1 } else { y2 };

        // The wing parameters are set on even (left) wings and reused for the
        // mirrored odd (right) wing.
        let (mut px, mut py, mut mpx, mut mpy) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        let mut bsl = 0;
        let mut battery_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        for i in 0..self.battery_num as usize {
            let mut wrl = 1;
            if i % 2 == 0 {
                px = enemy_size[5] + rand_next_float(enemy_size[6]);
                if self.battery_num <= 2 {
                    py = rand_next_signed_float(enemy_size[7]);
                } else if i < 2 {
                    py = rand_next_float(enemy_size[7] / 2.0) + enemy_size[7] / 2.0;
                } else {
                    py = -rand_next_float(enemy_size[7] / 2.0) - enemy_size[7] / 2.0;
                }
                let md = if rand_next_int(2) == 0 {
                    rand_next_float(PI / 2.0) - PI / 4.0
                } else {
                    rand_next_float(PI / 2.0) + PI / 4.0 * 3.0
                };
                mpx = px / 2.0
                    + md.sin() * (enemy_size[8] / 2.0 + rand_next_float(enemy_size[8] / 2.0));
                mpy = py / 2.0
                    + md.cos() * (enemy_size[8] / 2.0 + rand_next_float(enemy_size[8] / 2.0));
                match size {
                    TYPE_SMALL | TYPE_MIDDLE | TYPE_LARGE => bsl = 1,
                    TYPE_MIDDLEBOSS => bsl = 150 + rand_next_int(30),
                    TYPE_LARGEBOSS => bsl = 200 + rand_next_int(50),
                    _ => {}
                }
                battery_color = create_enemy_color(color_type);
                wrl = -1;
                if !self.wing_collision {
                    if px > self.collision_size.x {
                        self.collision_size.x = px;
                    }
                    let mut cpy = py.abs();
                    if cpy > self.collision_size.y {
                        self.collision_size.y = cpy;
                    }
                    cpy = mpy.abs();
                    if cpy > self.collision_size.y {
                        self.collision_size.y = cpy;
                    }
                }
            }
            let bt = &mut self.battery_type[i];
            bt.wing_shape_pos = create_wings(px, py, mpx, mpy, wrl);

            bt.collision_pos.x = (px + px / 4.0) / 2.0 * wrl as f32;
            bt.collision_pos.y = (py + mpy + py / 4.0) / 3.0;
            bt.collision_size.x = px / 4.0 * 3.0 / 2.0;
            let sy1 = (py - mpy).abs() / 2.0;
            let sy2 = (py - py / 4.0).abs() / 2.0;
            bt.collision_size.y = if sy1 > sy2 { sy1 } else { sy2 };
            bt.r = battery_color.r;
            bt.g = battery_color.g;
            bt.b = battery_color.b;
            bt.shield = bsl;
        }
    }

    // Set the barrage of a wing pair starting at batteryType[idx].
    #[allow(clippy::too_many_arguments)]
    fn set_battery(
        &mut self,
        rank: f32,
        n: i32,
        barrage_type: i32,
        barrage_intense: i32,
        idx: usize,
        ptn_idx: i32,
        slow: f32,
        mode: i32,
    ) {
        let (bt1, bt2) = set_battery_pair(
            self.battery_type[idx],
            self.battery_type[idx + 1],
            rank,
            n,
            barrage_type,
            barrage_intense,
            ptn_idx,
            slow,
            mode,
        );
        self.battery_type[idx] = bt1;
        self.battery_type[idx + 1] = bt2;
    }

    // Mirrors `firePeriod /= (2 - rank * 0.1)` applied below rank 10.
    fn ease_fire_period(&mut self, rank: f32) {
        if rank < 10.0 {
            self.fire_period = (self.fire_period as f64 / (2.0 - rank as f64 * 0.1)) as i32;
        }
    }

    fn set_small(&mut self, rank: f32, mode: i32) {
        self.kind = TYPE_SMALL;
        self.barrage_pattern_num = 1;
        self.wing_collision = false;
        let mut br = if mode == MODE_ROLL {
            barrage_create(CATEGORY_SMALL, mode)
        } else {
            barrage_create(CATEGORY_SMALL_LOCK, mode)
        };
        br = set_barrage_rank(br, rank, INTENSE_VERYWEAK, mode);
        br = set_barrage_shape(br, 0.7);
        br.x_reverse = (rand_next_int(2) * 2 - 1) as f32;
        self.barrage[0] = br;
        self.set_enemy_shape_and_wings(TYPE_SMALL);
        self.set_battery(0.0, 0, CATEGORY_MORPH, INTENSE_NORMAL, 0, 0, 1.0, mode);
        self.shield = 1;
        self.fire_interval = 99999;
        self.fire_period = 150 + rand_next_int(40);
        self.ease_fire_period(rank);
    }

    fn set_middle(&mut self, rank: f32, mode: i32) {
        self.kind = TYPE_MIDDLE;
        self.barrage_pattern_num = 1;
        self.wing_collision = false;
        let mut br = barrage_create(CATEGORY_MIDDLE, mode);
        let (cr, sr) = if mode == MODE_ROLL {
            match rand_next_int(6) {
                0 | 1 => (rank / 3.0 * 2.0, 0.0),
                2 => (rank / 4.0, rank / 4.0),
                _ => (0.0, rank / 2.0),
            }
        } else {
            match rand_next_int(6) {
                0 | 1 => (rank / 5.0, rank / 4.0),
                _ => (0.0, rank / 2.0),
            }
        };
        br = set_barrage_rank(br, cr, INTENSE_MORPHWEAK, mode);
        br = set_barrage_shape(br, 0.75);
        br.x_reverse = (rand_next_int(2) * 2 - 1) as f32;
        self.barrage[0] = br;
        self.set_enemy_shape_and_wings(TYPE_MIDDLE);
        if mode == MODE_ROLL {
            self.shield = 40 + rand_next_int(10);
            self.set_battery(sr, 1, CATEGORY_MIDDLESUB, INTENSE_NORMAL, 0, 0, 1.0, mode);
            self.fire_interval = 100 + rand_next_int(60);
            self.fire_period =
                (self.fire_interval as f64 / (1.8 + rand_next_float(0.7) as f64)) as i32;
        } else {
            self.shield = 30 + rand_next_int(8);
            self.set_battery(sr, 1, CATEGORY_MIDDLESUB_LOCK, INTENSE_NORMAL, 0, 0, 1.0, mode);
            self.fire_interval = 72 + rand_next_int(30);
            self.fire_period =
                (self.fire_interval as f64 / (1.2 + rand_next_float(0.2) as f64)) as i32;
        }
        self.ease_fire_period(rank);
    }

    fn set_large(&mut self, rank: f32, mode: i32) {
        self.kind = TYPE_LARGE;
        self.barrage_pattern_num = 1;
        self.wing_collision = false;
        let mut br = barrage_create(CATEGORY_LARGE, mode);
        let (cr, sr1, sr2) = if mode == MODE_ROLL {
            match rand_next_int(9) {
                0 | 1 | 2 | 3 => (rank, 0.0, 0.0),
                4 => (rank / 3.0 * 2.0, rank / 3.0 * 2.0, 0.0),
                5 => (rank / 3.0 * 2.0, 0.0, rank / 3.0 * 2.0),
                _ => (0.0, rank / 3.0 * 2.0, rank / 3.0 * 2.0),
            }
        } else {
            match rand_next_int(9) {
                0 => (rank / 4.0 * 3.0, 0.0, 0.0),
                1 | 2 => (rank / 4.0 * 2.0, rank / 3.0 * 2.0, 0.0),
                3 | 4 => (rank / 4.0 * 2.0, 0.0, rank / 3.0 * 2.0),
                _ => (0.0, rank / 3.0 * 2.0, rank / 3.0 * 2.0),
            }
        };
        br = set_barrage_rank(br, cr, INTENSE_WEAK, mode);
        br = set_barrage_shape(br, 0.8);
        br.x_reverse = (rand_next_int(2) * 2 - 1) as f32;
        self.barrage[0] = br;
        self.set_enemy_shape_and_wings(TYPE_LARGE);
        if mode == MODE_ROLL {
            self.shield = 60 + rand_next_int(10);
            self.set_battery(sr1, 1, CATEGORY_MIDDLESUB, INTENSE_NORMAL, 0, 0, 1.0, mode);
            self.set_battery(sr2, 1, CATEGORY_MIDDLESUB, INTENSE_NORMAL, 2, 0, 1.0, mode);
            self.fire_interval = 150 + rand_next_int(60);
            self.fire_period =
                (self.fire_interval as f64 / (1.3 + rand_next_float(0.8) as f64)) as i32;
        } else {
            self.shield = 45 + rand_next_int(8);
            self.set_battery(sr1, 1, CATEGORY_MIDDLESUB_LOCK, INTENSE_NORMAL, 0, 0, 1.0, mode);
            self.set_battery(sr2, 1, CATEGORY_MIDDLESUB_LOCK, INTENSE_NORMAL, 2, 0, 1.0, mode);
            self.fire_interval = 100 + rand_next_int(50);
            self.fire_period =
                (self.fire_interval as f64 / (1.2 + rand_next_float(0.2) as f64)) as i32;
        }
        self.ease_fire_period(rank);
    }

    fn set_middle_boss(&mut self, rank: f32, mode: i32) {
        self.kind = TYPE_MIDDLEBOSS;
        self.barrage_pattern_num = 2 + rand_next_int(2);
        self.wing_collision = true;
        let bn = 1 + rand_next_int(2);
        for i in 0..self.barrage_pattern_num {
            let mut br = barrage_create(CATEGORY_LARGE, mode);
            let (cr, sr) = match rand_next_int(3) {
                0 => (rank, 0.0),
                1 => (rank / 3.0, rank / 3.0),
                _ => (0.0, rank),
            };
            br = set_barrage_rank_slow(br, cr, INTENSE_NORMAL, mode, 0.9);
            br = set_barrage_shape(br, 0.9);
            br.x_reverse = (rand_next_int(2) * 2 - 1) as f32;
            self.barrage[i as usize] = br;
            self.set_enemy_shape_and_wings(TYPE_MIDDLEBOSS);
            self.set_battery(sr, bn, CATEGORY_MIDDLE, INTENSE_WEAK, 0, i, 0.9, mode);
        }
        self.shield = 300 + rand_next_int(50);
        self.fire_interval = 200 + rand_next_int(40);
        self.fire_period =
            (self.fire_interval as f64 / (1.2 + rand_next_float(0.4) as f64)) as i32;
        self.ease_fire_period(rank);
    }

    fn set_large_boss(&mut self, rank: f32, mode: i32) {
        self.kind = TYPE_LARGEBOSS;
        self.barrage_pattern_num = 2 + rand_next_int(3);
        self.wing_collision = true;
        let bn1 = 1 + rand_next_int(3);
        let bn2 = 1 + rand_next_int(3);
        for i in 0..self.barrage_pattern_num {
            let mut br = barrage_create(CATEGORY_LARGE, mode);
            let (cr, sr1, sr2) = match rand_next_int(3) {
                0 => (rank, 0.0, 0.0),
                1 => (rank / 3.0, rank / 3.0, 0.0),
                _ => (rank / 3.0, 0.0, rank / 3.0),
            };
            br = set_barrage_rank_slow(br, cr, INTENSE_NORMAL, mode, 0.9);
            br = set_barrage_shape(br, 1.0);
            br.x_reverse = (rand_next_int(2) * 2 - 1) as f32;
            self.barrage[i as usize] = br;
            self.set_enemy_shape_and_wings(TYPE_LARGEBOSS);
            self.set_battery(sr1, bn1, CATEGORY_MIDDLE, INTENSE_NORMAL, 0, i, 0.9, mode);
            self.set_battery(sr2, bn2, CATEGORY_MIDDLE, INTENSE_NORMAL, 2, i, 0.9, mode);
        }
        self.shield = 400 + rand_next_int(50);
        self.fire_interval = 220 + rand_next_int(60);
        self.fire_period =
            (self.fire_interval as f64 / (1.2 + rand_next_float(0.3) as f64)) as i32;
        self.ease_fire_period(rank);
    }
}

// stage_manager.rs (rust/src/stage_manager.rs) is the only caller.
pub fn enemy_type_create_small(id: c_int, rank: c_float, mode: c_int) -> EnemyType {
    let mut et = EnemyType::new(id);
    et.set_small(rank, mode);
    et
}

pub fn enemy_type_create_middle(id: c_int, rank: c_float, mode: c_int) -> EnemyType {
    let mut et = EnemyType::new(id);
    et.set_middle(rank, mode);
    et
}

pub fn enemy_type_create_large(id: c_int, rank: c_float, mode: c_int) -> EnemyType {
    let mut et = EnemyType::new(id);
    et.set_large(rank, mode);
    et
}

pub fn enemy_type_create_middle_boss(id: c_int, rank: c_float, mode: c_int) -> EnemyType {
    let mut et = EnemyType::new(id);
    et.set_middle_boss(rank, mode);
    et
}

pub fn enemy_type_create_large_boss(id: c_int, rank: c_float, mode: c_int) -> EnemyType {
    let mut et = EnemyType::new(id);
    et.set_large_boss(rank, mode);
    et
}
