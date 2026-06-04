use crate::rendering::gl::{
    glBegin, glEnd, glPopMatrix, glPushMatrix, glVertex3f, GL_TRIANGLE_FAN,
};

#[no_mangle]
pub extern "C" fn gl_begin_triangle_fan() {
    unsafe { glBegin(GL_TRIANGLE_FAN) };
}

#[no_mangle]
pub extern "C" fn gl_end() {
    unsafe { glEnd() };
}

#[no_mangle]
pub extern "C" fn gl_vertex_3f(x: f32, y: f32, z: f32) {
    unsafe { glVertex3f(x, y, z) };
}

#[no_mangle]
pub extern "C" fn gl_push_matrix() {
    unsafe { glPushMatrix() };
}

#[no_mangle]
pub extern "C" fn gl_pop_matrix() {
    unsafe { glPopMatrix() };
}
