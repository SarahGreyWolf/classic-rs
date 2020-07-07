use specs::{Component, VecStorage};

pub struct Pos {
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) z: i16,
}

impl Pos {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self {
            x,
            y,
            z,
        }
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Component for Pos {
    type Storage = VecStorage<Self>;
}