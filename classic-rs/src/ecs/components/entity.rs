use specs::{VecStorage, Component, FlaggedStorage};

#[derive(Default)]
pub struct DeltaVel {
    dx: i8,
    dy: i8,
    dz: i8,
}

impl Component for DeltaVel {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Default)]
pub struct Rotation {
    pub yaw: u8,
    pub pitch: u8,
}

impl Component for Rotation {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}