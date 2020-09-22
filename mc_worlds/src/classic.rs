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


/*      format_version: u8 - Constant, always at 1\
*       name: String - World Name\
*       uuid: Vec<u8> - Unique 128-bit world identifier\
*       x: i16 - width of the map\
*       y: i16 - height of the map\
*       z: i16 - length of the map\
*       created_by: Option<CreatedBy> - optional, identifies the creator of the map\
*       map_generator: Option<MapGenerator> - optional, contains data about map generation\
*       time_created: i64 - UTC Unix Timestamp of when the world was created\
*       last_accessed: i64 - UTC Unix Timestamp of when a player last accessed the world\
*       last_modified: i64 - UTC Unix Timestamp set when blocks are modified\
*       spawn: Spawn - Defines the point where the players spawn on the map\
*       block_array: Vec<u8> - The block data, 1 byte per block, same order as LevelDataChunk Packet\
*   }
*/
/// # ClassicWorld File Format
/// A file format for custom classic minecraft worlds defined by https://wiki.vg/ClassicWorld_file_format
pub struct ClassicWorld {
    /// Constant, always at 1
    format_version: u8,
    /// World Name
    name: String,
    /// Unique 128-bit world identifier
    uuid: [u8; 16],
    /// Width of the map
    x: i16,
    /// Height of the map
    y: i16,
    /// Length of the map
    z: i16,
    /// (optional) Identifies the creator of the map
    created_by: Option<CreatedBy>,
    /// (optional) Contains data about map generation
    map_generator: Option<MapGenerator>,
    /// UTC Unix Timestamp of when the world was created
    time_created: i64,
    /// UTC Unix Timestamp of when a player last accessed the world
    last_accessed: i64,
    /// UTC Unix Timestamp set when blocks are modified
    last_modified: i64,
    /// Defines the point where the players spawn on the map
    spawn: Spawn,
    /// The block data, 1 byte per block, same order as LevelDataChunk Packet
    block_array: Vec<u8>,
    // metadata: Vec<Metadata>
}

impl ClassicWorld {
    //! Create a new empty world with a name aswell as dimensions
    pub fn new(name: &str, x: i16, y: i16, z: i16) -> Self {
        Self {
            format_version: 1,
            name: name.to_string(),
            uuid: [0u8; 16],
            x,
            y,
            z,
            created_by: Some(CreatedBy { service: "".to_string(), username: "".to_string() }),
            map_generator: Some(MapGenerator { service: "".to_string(), username: "".to_string() }),
            time_created: 0,
            last_accessed: 0,
            last_modified: 0,
            spawn: Spawn {
                x: 0,
                y: 0,
                z: 0,
                h: 0,
                p: 0
            },
            block_array: (0..(x*y*z)).map(|x| 1).collect()
        }
    }

    pub fn get_size(&self) -> [i16; 3] {
        [self.x, self.y, self.z]
    }

    pub fn get_blocks(&self) -> &Vec<u8> {
        &self.block_array
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