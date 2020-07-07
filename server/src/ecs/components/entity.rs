use specs::{VecStorage, Component};

pub struct DeltaVel {
    dx: i8,
    dy: i8,
    dz: i8,
}

impl Default for DeltaVel {
    fn default() -> Self {
        Self {
            dx: 0,
            dy: 0,
            dz: 0,
        }
    }
}

impl Component for DeltaVel {
    type Storage = VecStorage<Self>;
}

pub struct Rotation {
    pub(crate) yaw: u8,
    pub(crate) pitch: u8,
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            yaw: 0,
            pitch: 0,
        }
    }
}

impl Component for Rotation {
    type Storage = VecStorage<Self>;
}