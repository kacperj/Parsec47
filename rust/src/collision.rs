#[repr(C)]
#[derive(Copy, Clone)]
pub struct CollisionBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

pub fn check_hit(b: CollisionBox, px: f32, py: f32) -> bool {
    px < b.x1 || px > b.x2 || py < b.y1 || py > b.y2
}

pub fn check_hit_with_space(b: CollisionBox, px: f32, py: f32, space: f32) -> bool {
    px < b.x1 + space || px > b.x2 - space || py < b.y1 + space || py > b.y2 - space
}
