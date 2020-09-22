use byteorder::{LittleEndian};

pub trait Metadata {

}

pub struct MineWorld {
    width: i32,
    height: i32,
    depth: i32,
    blocks: Vec<u8>,
    name: String,
    creator: String,
    create_time: u64,
    x_spawn: i32,
    y_spawn: i32,
    z_spawn: i32,
    rot_spawn: f32,
    water_level: i32,
    sky_color: i32,
    fog_color: i32,
    cloud_color: i32,
    tick_count: i32,
    grow_trees: bool
}

impl MineWorld {

}

pub struct ClassicWorld {
    format_version: u8,
    name: String,
    uuid: Vec<u8>,
    x: i16,
    y: i16,
    z: i16,
    created_by: CreatedBy,
    map_generator: MapGenerator,
    time_created: i64,
    last_access: i64,
    last_modified: i64,
    spawn: Spawn,
    block_array: Vec<u8>,
    // metadata: Vec<Metadata>
}

impl ClassicWorld {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self {
            format_version: 1,
            name: "".to_string(),
            uuid: vec![],
            x: 0,
            y: 0,
            z: 0,
            created_by: CreatedBy { service: "".to_string(), username: "".to_string() },
            map_generator: MapGenerator { service: "".to_string(), username: "".to_string() },
            time_created: 0,
            last_access: 0,
            last_modified: 0,
            spawn: Spawn {
                x: 0,
                y: 0,
                z: 0,
                h: 0,
                p: 0
            },
            block_array: vec![]
        }
    }
}

struct CreatedBy {
    service: String,
    username: String,
}

struct MapGenerator {
    service: String,
    username: String,
}

struct Spawn {
    x: i16,
    y: i16,
    z: i16,
    // Heading
    h: u8,
    // Pitch
    p: u8,
}