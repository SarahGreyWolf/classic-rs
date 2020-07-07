use specs::{Component, NullStorage, DenseVecStorage};

pub struct BlockType(u8);

impl Default for BlockType {
    fn default() -> Self {
        Self(0x00)
    }
}

impl Component for BlockType {
    type Storage = DenseVecStorage<Self>;
}