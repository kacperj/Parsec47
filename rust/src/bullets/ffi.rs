//! BulletML runner FFI — mirrors the C++ API in bulletml.dll. The two setters
//! getDefaultSpeed/getRand are declared (and their callbacks implemented) in
//! crate::barrage; everything else lives here so the bullets module owns runner
//! creation + callback registration.
use std::os::raw::c_void;

// Callback function-pointer shapes (= the C++ BML_fp_* typedefs). The runner
// passes itself as the first arg (a `BulletMLRunner*`), which our callbacks ignore.
pub type FpD = extern "C" fn(*mut c_void) -> f64;
pub type FpI = extern "C" fn(*mut c_void) -> i32;
pub type FpV = extern "C" fn(*mut c_void);
pub type FpVd = extern "C" fn(*mut c_void, f64);
pub type FpVdd = extern "C" fn(*mut c_void, f64, f64);
pub type FpVsdd = extern "C" fn(*mut c_void, *mut c_void, f64, f64);

// Duplicated under `#[cfg]` so the Windows-only `raw-dylib` kind is gated out on
// other targets (see barrage::mod for the rationale). Windows: raw-dylib. Linux:
// link libbulletml.so (build.rs supplies the search path + rpath).
#[cfg(windows)]
#[link(name = "bulletml", kind = "raw-dylib")]
extern "C" {
    pub fn BulletMLRunner_new_parser(parser: *mut c_void) -> *mut c_void;
    pub fn BulletMLRunner_new_state(state: *mut c_void) -> *mut c_void;
    pub fn BulletMLRunner_delete(runner: *mut c_void);
    pub fn BulletMLRunner_run(runner: *mut c_void);
    pub fn BulletMLRunner_isEnd(runner: *mut c_void) -> bool;

    pub fn BulletMLRunner_set_getBulletDirection(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getAimDirection(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getBulletSpeed(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getRank(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_createSimpleBullet(runner: *mut c_void, f: FpVdd);
    pub fn BulletMLRunner_set_createBullet(runner: *mut c_void, f: FpVsdd);
    pub fn BulletMLRunner_set_getTurn(runner: *mut c_void, f: FpI);
    pub fn BulletMLRunner_set_doVanish(runner: *mut c_void, f: FpV);
    pub fn BulletMLRunner_set_doChangeDirection(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_doChangeSpeed(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_doAccelX(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_doAccelY(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_getBulletSpeedX(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getBulletSpeedY(runner: *mut c_void, f: FpD);
}

#[cfg(not(windows))]
#[link(name = "bulletml")]
extern "C" {
    pub fn BulletMLRunner_new_parser(parser: *mut c_void) -> *mut c_void;
    pub fn BulletMLRunner_new_state(state: *mut c_void) -> *mut c_void;
    pub fn BulletMLRunner_delete(runner: *mut c_void);
    pub fn BulletMLRunner_run(runner: *mut c_void);
    pub fn BulletMLRunner_isEnd(runner: *mut c_void) -> bool;

    pub fn BulletMLRunner_set_getBulletDirection(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getAimDirection(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getBulletSpeed(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getRank(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_createSimpleBullet(runner: *mut c_void, f: FpVdd);
    pub fn BulletMLRunner_set_createBullet(runner: *mut c_void, f: FpVsdd);
    pub fn BulletMLRunner_set_getTurn(runner: *mut c_void, f: FpI);
    pub fn BulletMLRunner_set_doVanish(runner: *mut c_void, f: FpV);
    pub fn BulletMLRunner_set_doChangeDirection(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_doChangeSpeed(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_doAccelX(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_doAccelY(runner: *mut c_void, f: FpVd);
    pub fn BulletMLRunner_set_getBulletSpeedX(runner: *mut c_void, f: FpD);
    pub fn BulletMLRunner_set_getBulletSpeedY(runner: *mut c_void, f: FpD);
}
