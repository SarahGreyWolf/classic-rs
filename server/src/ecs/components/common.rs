use specs::{Component, VecStorage, FlaggedStorage};

#[derive(Default)]
pub struct Pos {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Component for Pos {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}