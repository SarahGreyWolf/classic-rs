use specs::{World, WorldExt};

pub mod components;
pub mod systems;

pub fn initialise_world() -> World {
    let mut world = World::new();
    world.register::<components::common::Pos>();
    world.register::<components::entity::DeltaVel>();
    world.register::<components::entity::Rotation>();
    world.register::<components::player::Player>();
    world.register::<components::player::Stream>();
    world
}