use crate::actors::actor_pool::ActorPool;
use crate::actors::fragment::Fragment;
use crate::actors::particle::Particle;
use crate::core::vector::Vector2;

static mut PARTICLE_POOL: Option<ActorPool<Particle>> = None;
static mut FRAGMENT_POOL: Option<ActorPool<Fragment>> = None;

fn get_particle_pool() -> &'static mut ActorPool<Particle> {
    unsafe { PARTICLE_POOL.get_or_insert_with(|| ActorPool::new(128, Particle::new)) }
}

#[no_mangle]
pub extern "C" fn particles_draw() {
    get_particle_pool().draw();
}

#[no_mangle]
pub extern "C" fn particles_draw_luminous() {
    get_particle_pool().draw_luminous();
}

#[no_mangle]
pub extern "C" fn particles_update() {
    get_particle_pool().update();
}

#[no_mangle]
pub extern "C" fn particles_init_new(x: f32, y: f32, d: f32, ofs: f32, speed: f32) {
    get_particle_pool().init_instance_force(|particle| {
        particle.init(Vector2 { x, y }, d, ofs, speed);
    });
}

fn get_fragment_pool() -> &'static mut ActorPool<Fragment> {
    unsafe { FRAGMENT_POOL.get_or_insert_with(|| ActorPool::new(128, Fragment::new)) }
}

#[no_mangle]
pub extern "C" fn fragments_draw() {
    get_fragment_pool().draw();
}

#[no_mangle]
pub extern "C" fn fragments_draw_luminous() {
    get_fragment_pool().draw_luminous();
}

#[no_mangle]
pub extern "C" fn fragments_update() {
    get_fragment_pool().update();
}

#[no_mangle]
pub extern "C" fn fragments_init_new(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    z: f32,
    speed: f32,
    deg: f32,
) {
    get_fragment_pool().init_instance_force(|fragment| {
        fragment.init(x1, y1, x2, y2, z, speed, deg);
    });
}
