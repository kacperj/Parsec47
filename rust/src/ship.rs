use core::ffi::c_float;

static mut SHIP_POS_X: c_float = 0.0;
static mut SHIP_POS_Y: c_float = 0.0;

#[no_mangle]
pub extern "C" fn ship_set_pos(x: c_float, y: c_float) {
    unsafe {
        SHIP_POS_X = x;
        SHIP_POS_Y = y;
    }
}

#[no_mangle]
pub extern "C" fn ship_get_pos_x() -> c_float {
    unsafe { SHIP_POS_X }
}

#[no_mangle]
pub extern "C" fn ship_get_pos_y() -> c_float {
    unsafe { SHIP_POS_Y }
}
