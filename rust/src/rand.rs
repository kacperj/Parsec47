const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908b0df;
const UMASK: u32 = 0x80000000;
const LMASK: u32 = 0x7fffffff;

static mut STATE: [u32; N] = [0; N];
static mut LEFT: i32 = 1;
static mut INITF: i32 = 0;
static mut NEXT_IDX: usize = 0;

fn mixbits(u: u32, v: u32) -> u32 {
    (u & UMASK) | (v & LMASK)
}

fn twist(u: u32, v: u32) -> u32 {
    (mixbits(u, v) >> 1) ^ if v & 1 != 0 { MATRIX_A } else { 0 }
}

#[no_mangle]
pub extern "C" fn init_genrand(s: u32) {
    unsafe {
        STATE[0] = s & 0xffffffff;
        for j in 1..N {
            STATE[j] = 1812433253u32
                .wrapping_mul(STATE[j - 1] ^ (STATE[j - 1] >> 30))
                .wrapping_add(j as u32);
            STATE[j] &= 0xffffffff;
        }
        LEFT = 1;
        INITF = 1;
    }
}

unsafe fn next_state() {
    if INITF == 0 {
        init_genrand(5489);
    }

    LEFT = N as i32;
    NEXT_IDX = 0;

    for j in 0..(N - M) {
        STATE[j] = STATE[j + M] ^ twist(STATE[j], STATE[j + 1]);
    }
    for j in (N - M)..(N - 1) {
        STATE[j] = STATE[j - (N - M)] ^ twist(STATE[j], STATE[j + 1]);
    }
    STATE[N - 1] = STATE[M - 1] ^ twist(STATE[N - 1], STATE[0]);
}

#[no_mangle]
pub extern "C" fn genrand_int32() -> u32 {
    unsafe {
        LEFT -= 1;
        if LEFT == 0 {
            next_state();
        }

        let mut y = STATE[NEXT_IDX];
        NEXT_IDX += 1;

        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;

        y
    }
}

#[no_mangle]
pub extern "C" fn genrand_real1() -> f64 {
    (genrand_int32() as f64) * (1.0 / 4294967295.0)
}

#[no_mangle]
pub extern "C" fn rand_set_seed(s: u32) {
    init_genrand(s);
}

#[no_mangle]
pub extern "C" fn rand_next_int(n: i32) -> i32 {
    (genrand_int32() % (n as u32)) as i32
}

#[no_mangle]
pub extern "C" fn rand_next_float(n: f32) -> f32 {
    (genrand_real1() * n as f64) as f32
}

#[no_mangle]
pub extern "C" fn rand_next_signed_float(n: f32) -> f32 {
    (genrand_real1() * (n as f64 * 2.0) - n as f64) as f32
}
