use specs::{World, WorldExt};

pub mod components;
pub mod systems;

pub fn initialise_world() -> World {
    let mut world = World::new();

    world
}