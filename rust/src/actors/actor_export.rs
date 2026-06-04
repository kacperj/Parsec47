use crate::actors::actor::Actor;
use crate::actors::actor_pool::ActorPool;
use crate::actors::fragment::Fragment;
use crate::actors::particle::Particle;
use crate::actors::roll::Roll;
use crate::actors::shot::Shot;
use crate::collision::CollisionBox;
use crate::core::vector::Vector2;
use crate::rendering::gl::*;

static mut PARTICLE_POOL: Option<ActorPool<Particle>> = None;
static mut FRAGMENT_POOL: Option<ActorPool<Fragment>> = None;
static mut SHOT_POOL: Option<ActorPool<Shot>> = None;

fn get_particle_pool() -> &'static mut ActorPool<Particle> {
    unsafe { PARTICLE_POOL.get_or_insert_with(|| ActorPool::new(128, Particle::new)) }
}

#[no_mangle]
pub extern "C" fn particles_draw() {
    unsafe { glBegin(GL_LINES) };
    get_particle_pool().draw();
    unsafe { glEnd() };
}

#[no_mangle]
pub extern "C" fn particles_draw_luminous() {
    unsafe { glBegin(GL_LINES) };
    get_particle_pool().draw_luminous();
    unsafe { glEnd() };
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

fn get_shot_pool() -> &'static mut ActorPool<Shot> {
    unsafe { SHOT_POOL.get_or_insert_with(|| ActorPool::new(32, Shot::new)) }
}

#[no_mangle]
pub extern "C" fn shots_update() {
    get_shot_pool().update();
}

#[no_mangle]
pub extern "C" fn shots_draw() {
    get_shot_pool().draw();
}

#[no_mangle]
pub extern "C" fn shots_draw_luminous() {}

#[no_mangle]
pub extern "C" fn shots_clear() {
    get_shot_pool().clear();
}

#[no_mangle]
pub extern "C" fn shots_init_new(x: f32, y: f32, deg: f32, bx1: f32, by1: f32, bx2: f32, by2: f32) {
    let field_box = CollisionBox { x1: bx1, y1: by1, x2: bx2, y2: by2 };
    if let Some(shot) = get_shot_pool().get_instance() {
        shot.init(x, y, deg, field_box);
    }
}

#[no_mangle]
pub extern "C" fn shots_is_active(i: i32) -> bool {
    get_shot_pool().actors[i as usize].is_active()
}

#[no_mangle]
pub extern "C" fn shots_get_pos_x(i: i32) -> f32 {
    get_shot_pool().actors[i as usize].pos.x
}

#[no_mangle]
pub extern "C" fn shots_get_pos_y(i: i32) -> f32 {
    get_shot_pool().actors[i as usize].pos.y
}

#[no_mangle]
pub extern "C" fn shots_set_inactive(i: i32) {
    get_shot_pool().actors[i as usize].set_active(false);
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
    unsafe { glBegin(GL_LINES) };
    get_fragment_pool().draw_luminous();
    unsafe { glEnd() };
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

static mut ROLL_POOL: Option<ActorPool<Roll>> = None;

fn get_roll_pool() -> &'static mut ActorPool<Roll> {
    unsafe { ROLL_POOL.get_or_insert_with(|| ActorPool::new(4, Roll::new)) }
}

#[no_mangle]
pub extern "C" fn rolls_clear() {
    get_roll_pool().clear();
}

#[no_mangle]
pub extern "C" fn rolls_draw() {
    get_roll_pool().draw();
}

#[no_mangle]
pub extern "C" fn rolls_init_new() {
    let x = crate::ship::ship_get_pos_x();
    let y = crate::ship::ship_get_pos_y();
    if let Some(roll) = get_roll_pool().get_instance() {
        roll.init(x, y);
    }
}

#[no_mangle]
pub extern "C" fn rolls_update() {
    let x = crate::ship::ship_get_pos_x();
    let y = crate::ship::ship_get_pos_y();
    for roll in get_roll_pool().actors.iter_mut() {
        if roll.is_active() {
            roll.tick(x, y, |px, py| {
                particles_init_new(px, py, core::f32::consts::PI, 0.8, 0.09375);
            });
        }
    }
}

#[no_mangle]
pub extern "C" fn rolls_release_all() {
    for roll in get_roll_pool().actors.iter_mut() {
        if roll.is_active() {
            roll.released = true;
        }
    }
}

#[no_mangle]
pub extern "C" fn rolls_pool_size() -> i32 {
    4
}

#[no_mangle]
pub extern "C" fn rolls_is_active(i: i32) -> bool {
    get_roll_pool().actors[i as usize].is_active()
}

#[no_mangle]
pub extern "C" fn rolls_get_pos0_x(i: i32) -> f32 {
    get_roll_pool().actors[i as usize].pos[0].x
}

#[no_mangle]
pub extern "C" fn rolls_get_pos0_y(i: i32) -> f32 {
    get_roll_pool().actors[i as usize].pos[0].y
}

#[no_mangle]
pub extern "C" fn rolls_is_released(i: i32) -> bool {
    get_roll_pool().actors[i as usize].released
}

#[no_mangle]
pub extern "C" fn rolls_get_cnt(i: i32) -> i32 {
    get_roll_pool().actors[i as usize].cnt
}
