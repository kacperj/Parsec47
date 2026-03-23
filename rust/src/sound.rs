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

static NO_SOUND: AtomicBool = AtomicBool::new(false);
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
    if NO_SOUND.load(Ordering::Relaxed) {
        return 0;
    }

    let sdl = match sdl2::init() {
        Ok(s) => s,
        Err(_) => {
            NO_SOUND.store(true, Ordering::Relaxed);
            return -1;
        }
    };

    let audio = match sdl.audio() {
        Ok(a) => a,
        Err(_) => {
            NO_SOUND.store(true, Ordering::Relaxed);
            return -1;
        }
    };

    if let Ok(ctx) = mixer::init(InitFlag::OGG) {
        std::mem::forget(ctx);
    }

    let result = unsafe {
        Mix_OpenAudioDevice(44100, AUDIO_S16LSB, 1, 4096, std::ptr::null(), 0)
    };
    if result < 0 {
        NO_SOUND.store(true, Ordering::Relaxed);
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
    if NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
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
pub extern "C" fn sound_set_no_sound(v: c_int) {
    NO_SOUND.store(v != 0, Ordering::Relaxed);
}

#[no_mangle]
pub extern "C" fn sound_get_no_sound() -> c_int {
    if NO_SOUND.load(Ordering::Relaxed) {
        1
    } else {
        0
    }
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
    if NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
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
    if NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
    unsafe {
        Mix_FadeOutChannel(MUSIC_CHANNEL, FADE_OUT_SPEED.load(Ordering::Relaxed));
    }
}

#[no_mangle]
pub extern "C" fn sound_stop_music() {
    if NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
    Channel(MUSIC_CHANNEL).halt();
}

#[no_mangle]
pub extern "C" fn sound_play_chunk(slot: c_int) {
    if NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
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

#[no_mangle]
pub extern "C" fn sound_halt_chunk(slot: c_int) {
    if NO_SOUND.load(Ordering::Relaxed) {
        return;
    }
    unsafe {
        if let Some(slots) = get_slots() {
            if let Some(s) = slots.get(slot as usize) {
                Channel(s.chunk_channel).halt();
            }
        }
    }
}
