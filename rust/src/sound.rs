use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, Ordering};

use sdl2::mixer::{self, Channel, Chunk, InitFlag};

const MAX_SLOTS: usize = 16;
const AUDIO_S16LSB: u16 = 0x8010;
const MUSIC_CHANNEL: i32 = 8;

extern "C" {
    fn Mix_OpenAudioDevice(
        frequency: c_int,
        format: u16,
        channels: c_int,
        chunksize: c_int,
        device: *const c_char,
        allowed_changes: c_int,
    ) -> c_int;
    fn Mix_FadeOutChannel(which: c_int, ms: c_int) -> c_int;
}

static FADE_OUT_SPEED: AtomicI32 = AtomicI32::new(1280);
static SLOTS: AtomicPtr<Vec<SoundSlot>> = AtomicPtr::new(std::ptr::null_mut());

struct SoundSlot {
    music_chunk: Option<Chunk>,
    chunk: Option<Chunk>,
    chunk_channel: i32,
}

unsafe fn get_slots() -> Option<&'static mut Vec<SoundSlot>> {
    let ptr = SLOTS.load(Ordering::Relaxed);
    if ptr.is_null() {
        None
    } else {
        Some(&mut *ptr)
    }
}

#[no_mangle]
pub extern "C" fn sound_init() -> c_int {
    let sdl = match sdl2::init() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let audio = match sdl.audio() {
        Ok(a) => a,
        Err(_) => return -1,
    };

    if let Ok(ctx) = mixer::init(InitFlag::OGG) {
        std::mem::forget(ctx);
    }

    let result = unsafe { Mix_OpenAudioDevice(44100, AUDIO_S16LSB, 1, 4096, std::ptr::null(), 0) };
    if result < 0 {
        return -1;
    }

    // 8 SE channels (0-7) + 1 music channel (8)
    mixer::allocate_channels(MUSIC_CHANNEL + 1);

    std::mem::forget(sdl);
    std::mem::forget(audio);

    let slots = Box::into_raw(Box::new(Vec::<SoundSlot>::with_capacity(MAX_SLOTS)));
    SLOTS.store(slots, Ordering::Relaxed);

    0
}

#[no_mangle]
pub extern "C" fn sound_close() {
    Channel(MUSIC_CHANNEL).halt();
    let ptr = SLOTS.swap(std::ptr::null_mut(), Ordering::Relaxed);
    if !ptr.is_null() {
        unsafe {
            drop(Box::from_raw(ptr));
        }
    }
    mixer::close_audio();
}

#[no_mangle]
pub extern "C" fn sound_set_fade_out_speed(speed: c_int) {
    FADE_OUT_SPEED.store(speed, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn sound_alloc_slot() -> c_int {
    unsafe {
        match get_slots() {
            Some(slots) => {
                let idx = slots.len();
                if idx >= MAX_SLOTS {
                    return -1;
                }
                slots.push(SoundSlot {
                    music_chunk: None,
                    chunk: None,
                    chunk_channel: 0,
                });
                idx as c_int
            }
            None => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_load_music(slot: c_int, path: *const c_char) -> c_int {
    if path.is_null() {
        return -1;
    }
    unsafe {
        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        let chunk = match Chunk::from_file(Path::new(path_str)) {
            Ok(c) => c,
            Err(_) => return -1,
        };
        match get_slots() {
            Some(slots) => match slots.get_mut(slot as usize) {
                Some(s) => {
                    s.music_chunk = Some(chunk);
                    0
                }
                None => -1,
            },
            None => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_load_chunk(slot: c_int, path: *const c_char, channel: c_int) -> c_int {
    if path.is_null() {
        return -1;
    }
    unsafe {
        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        let chunk = match Chunk::from_file(Path::new(path_str)) {
            Ok(c) => c,
            Err(_) => return -1,
        };
        match get_slots() {
            Some(slots) => match slots.get_mut(slot as usize) {
                Some(s) => {
                    s.chunk = Some(chunk);
                    s.chunk_channel = channel;
                    0
                }
                None => -1,
            },
            None => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_free_slot(slot: c_int) {
    unsafe {
        if let Some(slots) = get_slots() {
            if let Some(s) = slots.get_mut(slot as usize) {
                if s.music_chunk.is_some() {
                    Channel(MUSIC_CHANNEL).halt();
                    s.music_chunk = None;
                }
                if s.chunk.is_some() {
                    Channel(s.chunk_channel).halt();
                    s.chunk = None;
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_play_music(slot: c_int) {
    unsafe {
        if let Some(slots) = get_slots() {
            if let Some(s) = slots.get(slot as usize) {
                if let Some(ref chunk) = s.music_chunk {
                    let _ = Channel(MUSIC_CHANNEL).play(chunk, -1);
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_fade_music() {
    unsafe {
        Mix_FadeOutChannel(MUSIC_CHANNEL, FADE_OUT_SPEED.load(Ordering::Relaxed));
    }
}

#[no_mangle]
pub extern "C" fn sound_stop_music() {
    Channel(MUSIC_CHANNEL).halt();
}

#[no_mangle]
pub extern "C" fn sound_play_chunk(slot: c_int) {
    unsafe {
        if let Some(slots) = get_slots() {
            if let Some(s) = slots.get(slot as usize) {
                if let Some(ref chunk) = s.chunk {
                    let _ = Channel(s.chunk_channel).play(chunk, 0);
                }
            }
        }
    }
}

// ── SoundManager layer ────────────────────────────────────────────────────

static SM_NO_SOUND: AtomicBool = AtomicBool::new(false);
static SM_IS_IN_GAME: AtomicBool = AtomicBool::new(false);

const BGM_COUNT: usize = 4;
const SE_COUNT: usize = 11;

const BGM_FILES: [&str; BGM_COUNT] = [
    "assets/sounds/ptn0.ogg",
    "assets/sounds/ptn1.ogg",
    "assets/sounds/ptn2.ogg",
    "assets/sounds/ptn3.ogg",
];

const SE_FILES: [&str; SE_COUNT] = [
    "assets/sounds/shot.wav",
    "assets/sounds/rollchg.wav",
    "assets/sounds/rollrls.wav",
    "assets/sounds/shipdst.wav",
    "assets/sounds/getbonus.wav",
    "assets/sounds/extend.wav",
    "assets/sounds/enemydst.wav",
    "assets/sounds/largedst.wav",
    "assets/sounds/bossdst.wav",
    "assets/sounds/lock.wav",
    "assets/sounds/laser.wav",
];

const SE_CHANNELS: [i32; SE_COUNT] = [0, 1, 2, 1, 3, 4, 5, 6, 7, 1, 2];

static SM_BGM_SLOTS: [AtomicI32; BGM_COUNT] = [
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
];

static SM_SE_SLOTS: [AtomicI32; SE_COUNT] = [
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
    AtomicI32::new(-1),
];

#[no_mangle]
pub extern "C" fn sound_manager_set_no_sound(v: c_int) {
    SM_NO_SOUND.store(v != 0, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn sound_manager_set_in_game(v: c_int) {
    SM_IS_IN_GAME.store(v != 0, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn sound_manager_init() -> c_int {
    if SM_NO_SOUND.load(Ordering::Relaxed) {
        return 0;
    }
    sound_set_fade_out_speed(1280);
    if sound_init() < 0 {
        SM_NO_SOUND.store(true, Ordering::Relaxed);
        return -1;
    }
    for i in 0..BGM_COUNT {
        let slot = sound_alloc_slot();
        if slot < 0 {
            SM_NO_SOUND.store(true, Ordering::Relaxed);
            return -1;
        }
        let path = std::ffi::CString::new(BGM_FILES[i]).unwrap();
        if sound_load_music(slot, path.as_ptr()) < 0 {
            SM_NO_SOUND.store(true, Ordering::Relaxed);
            return -1;
        }
        SM_BGM_SLOTS[i].store(slot, Ordering::Relaxed);
    }
    for i in 0..SE_COUNT {
        let slot = sound_alloc_slot();
        if slot < 0 {
            SM_NO_SOUND.store(true, Ordering::Relaxed);
            return -1;
        }
        let path = std::ffi::CString::new(SE_FILES[i]).unwrap();
        if sound_load_chunk(slot, path.as_ptr(), SE_CHANNELS[i]) < 0 {
            SM_NO_SOUND.store(true, Ordering::Relaxed);
            return -1;
        }
        SM_SE_SLOTS[i].store(slot, Ordering::Relaxed);
    }
    0
}

#[no_mangle]
pub extern "C" fn sound_manager_close() {
    if SM_NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
    for slot_atom in &SM_BGM_SLOTS {
        let slot = slot_atom.swap(-1, Ordering::Relaxed);
        if slot >= 0 {
            sound_free_slot(slot);
        }
    }
    for slot_atom in &SM_SE_SLOTS {
        let slot = slot_atom.swap(-1, Ordering::Relaxed);
        if slot >= 0 {
            sound_free_slot(slot);
        }
    }
    sound_close();
}

#[no_mangle]
pub extern "C" fn sound_manager_play_bgm(n: c_int) {
    if SM_NO_SOUND.load(Ordering::Relaxed) || !SM_IS_IN_GAME.load(Ordering::Relaxed) {
        return;
    }
    if let Some(slot_atom) = SM_BGM_SLOTS.get(n as usize) {
        let slot = slot_atom.load(Ordering::Relaxed);
        if slot >= 0 {
            sound_play_music(slot);
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_manager_play_se(n: c_int) {
    if SM_NO_SOUND.load(Ordering::Relaxed) || !SM_IS_IN_GAME.load(Ordering::Relaxed) {
        return;
    }
    if let Some(slot_atom) = SM_SE_SLOTS.get(n as usize) {
        let slot = slot_atom.load(Ordering::Relaxed);
        if slot >= 0 {
            sound_play_chunk(slot);
        }
    }
}

#[no_mangle]
pub extern "C" fn sound_manager_fade_music() {
    if SM_NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
    sound_fade_music();
}

#[no_mangle]
pub extern "C" fn sound_manager_stop_music() {
    if SM_NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
    sound_stop_music();
}
