pub mod barrage_export;

use crate::core::rand::{rand_next_float, rand_next_int, rand_next_signed_float};
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;

// BulletML parser functions from the C++ bulletml library. The `raw-dylib` link
// kind is Windows-only and rejected outright on other targets, and it must sit
// on the extern block itself (it synthesizes import stubs from these symbols) —
// so the block is duplicated under `#[cfg]` rather than gated with `cfg_attr`.
// Windows: raw-dylib, no import lib needed. Linux: link libbulletml.so (build.rs
// supplies the search path + rpath).
#[cfg(windows)]
#[link(name = "bulletml", kind = "raw-dylib")]
extern "C" {
    fn BulletMLParserTinyXML_new(path: *const c_char) -> *mut c_void;
    fn BulletMLParserTinyXML_parse(parser: *mut c_void);
    fn BulletMLParserTinyXML_delete(parser: *mut c_void);
    pub(crate) fn BulletMLRunner_set_getDefaultSpeed(
        runner: *mut c_void,
        f: extern "C" fn(*mut c_void) -> f64,
    );
    pub(crate) fn BulletMLRunner_set_getRand(
        runner: *mut c_void,
        f: extern "C" fn(*mut c_void) -> f64,
    );
}

#[cfg(not(windows))]
#[link(name = "bulletml")]
extern "C" {
    fn BulletMLParserTinyXML_new(path: *const c_char) -> *mut c_void;
    fn BulletMLParserTinyXML_parse(parser: *mut c_void);
    fn BulletMLParserTinyXML_delete(parser: *mut c_void);
    pub(crate) fn BulletMLRunner_set_getDefaultSpeed(
        runner: *mut c_void,
        f: extern "C" fn(*mut c_void) -> f64,
    );
    pub(crate) fn BulletMLRunner_set_getRand(
        runner: *mut c_void,
        f: extern "C" fn(*mut c_void) -> f64,
    );
}

pub const BARRAGE_TYPE: usize = 13;
pub const BARRAGE_MAX: usize = 64;
pub const MORPH_MAX: usize = 8; // = Bullet.MORPH_MAX (Bullet.d)

// Category indices — must match the BarrageManager.d enum (== DIR_NAME order).
pub const CATEGORY_MORPH: i32 = 0;
pub const CATEGORY_SMALL: i32 = 1;
pub const CATEGORY_SMALLMOVE: i32 = 2;
pub const CATEGORY_SMALLSIDEMOVE: i32 = 3;
pub const CATEGORY_MIDDLE: i32 = 4;
pub const CATEGORY_MIDDLESUB: i32 = 5;
pub const CATEGORY_MIDDLEMOVE: i32 = 6;
pub const CATEGORY_LARGE: i32 = 8;
pub const CATEGORY_LARGEMOVE: i32 = 9;
pub const CATEGORY_MORPH_LOCK: i32 = 10;
pub const CATEGORY_SMALL_LOCK: i32 = 11;
pub const CATEGORY_MIDDLESUB_LOCK: i32 = 12;

pub const MODE_ROLL: i32 = 0; // = ShipMode.ROLL (ShipMode.d)

// Barrage intense levels.
pub const INTENSE_NORMAL: i32 = 0;
pub const INTENSE_WEAK: i32 = 1;
pub const INTENSE_VERYWEAK: i32 = 2;
pub const INTENSE_MORPHWEAK: i32 = 3;

const BULLET_SHAPE_NUM: i32 = 7;
const BULLET_COLOR_NUM: i32 = 4;

/// Barrage pattern parameters.
/// Canonical definition (formerly mirrored as `struct Barrage` in EnemyType.d);
/// built and consumed entirely within Rust.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Barrage {
    pub parser: *mut c_void,
    pub morph_parser: [*mut c_void; MORPH_MAX],
    pub morph_num: i32,
    pub morph_cnt: i32,
    pub rank: f32,
    pub speed_rank: f32,
    pub morph_rank: f32,
    pub shape: i32,
    pub color: i32,
    pub bullet_size: f32,
    pub x_reverse: f32,
}

impl Barrage {
    pub const fn empty() -> Self {
        Barrage {
            parser: ptr::null_mut(),
            morph_parser: [ptr::null_mut(); MORPH_MAX],
            morph_num: 0,
            morph_cnt: 0,
            rank: 0.0,
            speed_rank: 0.0,
            morph_rank: 0.0,
            shape: 0,
            color: 0,
            bullet_size: 0.0,
            x_reverse: 0.0,
        }
    }
}

// Mirrors EnemyType.setBarrageRank: derives the barrage difficulty knobs
// (rank, speed rank, morph rank/count) from the stage rank, then tones them
// down according to the intense level.
pub fn set_barrage_rank(mut br: Barrage, mut rank: f32, intense: i32, mode: i32) -> Barrage {
    if rank <= 0.0 {
        br.rank = 0.0;
        return br;
    }
    br.rank = rank.sqrt() / (8 - rand_next_int(3)) as f32;
    if br.rank > 0.8 {
        br.rank = rand_next_float(0.2) + 0.8;
    }
    rank /= br.rank + 2.0;
    if intense == INTENSE_WEAK {
        br.rank /= 2.0;
    }
    if mode == MODE_ROLL {
        br.speed_rank = rank.sqrt() * (rand_next_float(0.2) + 1.0);
    } else {
        br.speed_rank = (rank * 0.66).sqrt() * (rand_next_float(0.2) + 0.8);
    }
    if br.speed_rank < 1.0 {
        br.speed_rank = 1.0;
    }
    if br.speed_rank > 2.0 {
        br.speed_rank = br.speed_rank.sqrt() + 0.27;
    }
    br.morph_rank = rank / br.speed_rank;
    br.morph_cnt = 0;
    while br.morph_rank > 1.0 {
        br.morph_cnt += 1;
        br.morph_rank /= 3.0;
    }
    if intense == INTENSE_VERYWEAK {
        br.morph_rank /= 2.0;
        br.morph_cnt = (br.morph_cnt as f32 / 1.7) as i32;
    } else if intense == INTENSE_MORPHWEAK {
        br.morph_rank /= 2.0;
    } else if intense == INTENSE_WEAK {
        br.morph_rank /= 1.5;
    }
    br
}

// Mirrors EnemyType.setBarrageRankSlow.
pub fn set_barrage_rank_slow(br: Barrage, rank: f32, intense: i32, mode: i32, slow: f32) -> Barrage {
    let mut br = set_barrage_rank(br, rank, intense, mode);
    br.speed_rank *= slow;
    br
}

// Mirrors EnemyType.setBarrageShape: picks a random bullet shape/color and
// jitters the bullet size around the given base size.
pub fn set_barrage_shape(mut br: Barrage, size: f32) -> Barrage {
    br.shape = rand_next_int(BULLET_SHAPE_NUM);
    br.color = rand_next_int(BULLET_COLOR_NUM);
    br.bullet_size = (1.0 + rand_next_signed_float(0.1)) * size;
    br
}

const DIR_NAME: [&str; BARRAGE_TYPE] = [
    "morph",
    "small",
    "smallmove",
    "smallsidemove",
    "middle",
    "middlesub",
    "middlemove",
    "middlebackmove",
    "large",
    "largemove",
    "morph_lock",
    "small_lock",
    "middlesub_lock",
];

pub struct BarrageManager {
    parsers: [[*mut c_void; BARRAGE_MAX]; BARRAGE_TYPE],
    parser_num: [i32; BARRAGE_TYPE],
}

impl BarrageManager {
    pub const fn new() -> Self {
        BarrageManager {
            parsers: [[ptr::null_mut(); BARRAGE_MAX]; BARRAGE_TYPE],
            parser_num: [0; BARRAGE_TYPE],
        }
    }

    pub fn load_bulletmls(&mut self) {
        for (i, dir_name) in DIR_NAME.iter().enumerate() {
            let dir = format!("assets/bulletdata/{}", dir_name);
            let mut file_names: Vec<String> = std::fs::read_dir(&dir)
                .unwrap_or_else(|e| panic!("cannot read BulletML dir {}: {}", dir, e))
                .filter_map(|entry| {
                    let path = entry.ok()?.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("xml") {
                        Some(path.file_name()?.to_str()?.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            // Sort for a deterministic index -> file mapping (read_dir order is OS-dependent).
            file_names.sort();

            let mut j = 0;
            for file_name in file_names {
                if j >= BARRAGE_MAX {
                    break;
                }
                let path = format!("{}/{}", dir, file_name);
                eprintln!("Load BulletML: {}", path);
                let c_path = CString::new(path).unwrap();
                unsafe {
                    let parser = BulletMLParserTinyXML_new(c_path.as_ptr());
                    BulletMLParserTinyXML_parse(parser);
                    self.parsers[i][j] = parser;
                }
                j += 1;
            }
            self.parser_num[i] = j as i32;
        }
    }

    pub fn get_move_parser(&self, category: i32, move_type_random: i32) -> *mut c_void {
        let num = self.get_parser_num(category);
        if num <= 0 {
            return ptr::null_mut();
        }
        self.parsers[category as usize][(move_type_random % num) as usize]
    }

    pub fn get_parser_num(&self, category: i32) -> i32 {
        self.parser_num[category as usize]
    }

    // Mirrors EnemyType.setBarrageType: picks the move parser and a set of
    // distinct morph parsers for a fresh barrage. Remaining fields start
    // zeroed; the rank/shape setters fill them in afterwards.
    pub fn create_barrage(&self, btn: i32, mode: i32) -> Barrage {
        let mut br = Barrage::empty();

        let barrage_type_random = rand_next_int(i32::MAX);
        br.parser = self.get_move_parser(btn, barrage_type_random);

        // The D original reset a static array at every call; a local is equivalent.
        let mut used_morph_parser = [false; BARRAGE_MAX];

        let morph_parser_category = if mode == MODE_ROLL {
            CATEGORY_MORPH
        } else {
            CATEGORY_MORPH_LOCK
        };

        let available_morph_parsers = self.get_parser_num(morph_parser_category);

        for i in 0..MORPH_MAX {
            let mi = get_unused_morph_index(&used_morph_parser, available_morph_parsers);
            used_morph_parser[mi] = true;

            br.morph_parser[i] = self.get_move_parser(morph_parser_category, mi as i32);
        }
        br.morph_num = MORPH_MAX as i32;

        br
    }

    pub fn unload_bulletmls(&mut self) {
        for i in 0..BARRAGE_TYPE {
            for j in 0..self.parser_num[i] as usize {
                unsafe {
                    BulletMLParserTinyXML_delete(self.parsers[i][j]);
                }
                self.parsers[i][j] = ptr::null_mut();
            }
            self.parser_num[i] = 0;
        }
    }
}

// Mirrors EnemyType.getUnusedMorphIndex: random start, linear probe with
// wraparound; gives up (returning a used index) after one full pass.
fn get_unused_morph_index(used_morph_parser: &[bool; BARRAGE_MAX], available_morph_parsers: i32) -> usize {
    let mut mi = rand_next_int(available_morph_parsers) as usize;

    for _ in 0..available_morph_parsers {
        if !used_morph_parser[mi] {
            break;
        }
        mi += 1;
        if mi >= available_morph_parsers as usize {
            mi = 0;
        }
    }
    mi
}
