use std::net::TcpStream;
use specs::{Component, DenseVecStorage};
use specs::storage::{VecStorage};

pub struct Stream(pub TcpStream);

impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        Self(stream)
    }
}

impl Component for Stream {
    type Storage = VecStorage<Self>;
}

pub struct Player {
    pub username: String,
    pub ver_key: String,
    pub user_type: u8,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            ver_key: "".to_string(),
            user_type: 0x00
        }
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}