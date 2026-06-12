//! BulletML-driven bullet system: a pool of bullets, the callbacks the C++
//! BulletML engine calls back into, and the `bullets_*` API used by the rest of
//! the game. Port of src/abagames/p47/bullets/. (Rendering lives in crate::bullet_actor.)
pub mod bullet;
pub mod bullet_actor;
pub mod bullet_actor_pool;
pub mod ffi;
