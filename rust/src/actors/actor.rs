pub trait Actor {
    fn update(&mut self);
    fn draw(&self);
    fn draw_luminous(&self);
    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);
}
