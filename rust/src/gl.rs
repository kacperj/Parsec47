use core::ffi::c_float;

pub type GLenum = u32;
pub type GLint = i32;
pub type GLuint = u32;
pub type GLsizei = i32;

pub const GL_LINES: GLenum = 0x0001;
pub const GL_LINE_LOOP: GLenum = 0x0002;
pub const GL_QUADS: GLenum = 0x0007;
pub const GL_TRIANGLE_FAN: GLenum = 0x0006;
pub const GL_COMPILE: GLenum = 0x1300;
pub const GL_BLEND: GLenum = 0x0BE2;
pub const GL_TEXTURE_2D: GLenum = 0x0DE1;
pub const GL_BGR: GLenum = 0x80E0;
pub const GL_UNSIGNED_BYTE: GLenum = 0x1401;
pub const GL_TEXTURE_MIN_FILTER: GLenum = 0x2801;
pub const GL_TEXTURE_MAG_FILTER: GLenum = 0x2800;
pub const GL_LINEAR: GLint = 0x2601;

#[link(name = "opengl32")]
extern "system" {
    pub fn glColor4f(red: c_float, green: c_float, blue: c_float, alpha: c_float);
    pub fn glBegin(mode: GLenum);
    pub fn glEnd();
    pub fn glVertex3f(x: c_float, y: c_float, z: c_float);
    pub fn glPushMatrix();
    pub fn glPopMatrix();
    pub fn glTranslatef(x: c_float, y: c_float, z: c_float);
    pub fn glScalef(x: c_float, y: c_float, z: c_float);
    pub fn glRotatef(angle: c_float, x: c_float, y: c_float, z: c_float);
    pub fn glCallList(list: GLuint);
    pub fn glGenLists(range: GLsizei) -> GLuint;
    pub fn glNewList(list: GLuint, mode: GLenum);
    pub fn glEndList();
    pub fn glDeleteLists(list: GLuint, range: GLsizei);
    pub fn glEnable(cap: GLenum);
    pub fn glDisable(cap: GLenum);
    pub fn glGenTextures(n: GLsizei, textures: *mut GLuint);
    pub fn glBindTexture(target: GLenum, texture: GLuint);
    pub fn glTexImage2D(target: GLenum, level: GLint, internalformat: GLint,
        width: GLsizei, height: GLsizei, border: GLint,
        format: GLenum, type_: GLenum, pixels: *const u8);
    pub fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);
    pub fn glDeleteTextures(n: GLsizei, textures: *const GLuint);
    pub fn glTexCoord2f(s: c_float, t: c_float);
}
