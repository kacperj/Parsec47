use crate::actors::actor::Actor;

pub struct ActorPool<T: Actor> {
    pub actors: Vec<T>,
    current_index: i32,
}

impl<T: Actor> ActorPool<T> {
    pub fn new(capacity: i32, factory: impl Fn() -> T) -> ActorPool<T> {
        let mut actors: Vec<T> = Vec::with_capacity(capacity as usize);
        for _ in 0..capacity {
            actors.push(factory());
        }

        ActorPool {
            actors,
            current_index: capacity,
        }
    }

    pub fn update(&mut self) {
        for i in 0..self.actors.len() {
            if self.actors[i].is_active() {
                self.actors[i].update();
            }
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.actors.len() {
            self.actors[i].set_active(false);
        }
    }

    pub fn draw(&self) {
        for actor in &self.actors {
            if actor.is_active() {
                actor.draw();
            }
        }
    }

    pub fn draw_luminous(&self) {
        for actor in &self.actors {
            if actor.is_active() {
                actor.draw_luminous();
            }
        }
    }

    pub fn init_instance_force(&mut self, factory: impl Fn(&mut T)) {
        self.current_index -= 1;
        if self.current_index < 0 {
            self.current_index = self.actors.len() as i32 - 1;
        }

        let actor = &mut self.actors[self.current_index as usize];
        factory(actor);
    }
}
