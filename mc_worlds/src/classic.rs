use byteorder::{LittleEndian};

#[derive(Debug, Copy, Clone)]
pub enum Block {
    Air,
    Stone,
    GrassBlock,
    Dirt,
    Cobblestone,
    Planks,
    Sapling,
    Bedrock,
    FlowingWater,
    StationaryWater,
    FlowingLava,
    StationaryLava,
    Sand,
    Gravel,
    GoldOre,
    IronOre,
    CoalOre,
    Wood,
    Leaves,
    Sponge,
    Glass,
    RedCloth,
    OrangeCloth,
    YellowCloth,
    ChartreuseCloth,
    GreenCloth,
    SpringGreenCloth,
    CyanCloth,
    CapriCloth,
    UltramarineCloth,
    VioletCloth,
    PurpleCloth,
    MagentaCloth,
    RoseCloth,
    DarkGrayCloth,
    LightGrayCloth,
    WhiteCloth,
    Dandelion,
    Rose,
    BrownMushroom,
    RedMushroom,
    GoldBlock,
    IronBlock,
    DoubleSlab,
    Slab,
    Bricks,
    TNT,
    Bookshelf,
    MossyCobblestone,
    Obsidian
}

impl From<u8> for Block {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => Self::Air,
            0x01 => Self::Stone,
            0x02 => Self::GrassBlock,
            0x03 => Self::Dirt,
            0x04 => Self::Cobblestone,
            0x05 => Self::Planks,
            0x06 => Self::Sapling,
            0x07 => Self::Bedrock,
            0x08 => Self::FlowingWater,
            0x09 => Self::StationaryWater,
            0x0A => Self::FlowingLava,
            0x0B => Self::StationaryLava,
            0x0C => Self::Sand,
            0x0D => Self::Gravel,
            0x0E => Self::GoldOre,
            0x0F => Self::IronOre,
            0x10 => Self::CoalOre,
            0x11 => Self::Wood,
            0x12 => Self::Leaves,
            0x13 => Self::Sponge,
            0x14 => Self::Glass,
            0x15 => Self::RedCloth,
            0x16 => Self::OrangeCloth,
            0x17 => Self::YellowCloth,
            0x18 => Self::ChartreuseCloth,
            0x19 => Self::GreenCloth,
            0x1A => Self::SpringGreenCloth,
            0x1B => Self::CyanCloth,
            0x1C => Self::CapriCloth,
            0x1D => Self::UltramarineCloth,
            0x1E => Self::VioletCloth,
            0x1F => Self::PurpleCloth,
            0x20 => Self::MagentaCloth,
            0x21 => Self::RoseCloth,
            0x22 => Self::DarkGrayCloth,
            0x23 => Self::LightGrayCloth,
            0x24 => Self::WhiteCloth,
            0x25 => Self::Dandelion,
            0x26 => Self::Rose,
            0x27 => Self::BrownMushroom,
            0x28 => Self::RedMushroom,
            0x29 => Self::GoldBlock,
            0x2A => Self::IronBlock,
            0x2B => Self::DoubleSlab,
            0x2C => Self::Slab,
            0x2D => Self::Bricks,
            0x2E => Self::TNT,
            0x2F => Self::Bookshelf,
            0x30 => Self::MossyCobblestone,
            0x31 => Self::Obsidian,
            _ => Self::Air
        }
    }
}

impl From<Block> for u8 {
    fn from(block: Block) -> Self {
        match block {
            Block::Air => 0x00,
            Block::Stone => 0x01,
            Block::GrassBlock => 0x02,
            Block::Dirt => 0x03,
            Block::Cobblestone => 0x04,
            Block::Planks => 0x05,
            Block::Sapling => 0x06,
            Block::Bedrock => 0x07,
            Block::FlowingWater => 0x08,
            Block::StationaryWater => 0x09,
            Block::FlowingLava => 0x0A,
            Block::StationaryLava => 0x0B,
            Block::Sand => 0x0C,
            Block::Gravel => 0x0D,
            Block::GoldOre => 0x0E,
            Block::IronOre => 0x0F,
            Block::CoalOre => 0x10,
            Block::Wood => 0x11,
            Block::Leaves => 0x12,
            Block::Sponge => 0x13,
            Block::Glass => 0x14,
            Block::RedCloth => 0x15,
            Block::OrangeCloth => 0x16,
            Block::YellowCloth => 0x17,
            Block::ChartreuseCloth => 0x18,
            Block::GreenCloth => 0x19,
            Block::SpringGreenCloth => 0x1A,
            Block::CyanCloth => 0x1B,
            Block::CapriCloth => 0x1C,
            Block::UltramarineCloth => 0x1D,
            Block::VioletCloth => 0x1E,
            Block::PurpleCloth => 0x1F,
            Block::MagentaCloth => 0x20,
            Block::RoseCloth => 0x21,
            Block::DarkGrayCloth => 0x22,
            Block::LightGrayCloth => 0x23,
            Block::WhiteCloth => 0x24,
            Block::Dandelion => 0x25,
            Block::Rose => 0x26,
            Block::BrownMushroom => 0x27,
            Block::RedMushroom => 0x28,
            Block::GoldBlock => 0x29,
            Block::IronBlock => 0x2A,
            Block::DoubleSlab => 0x2B,
            Block::Slab => 0x2C,
            Block::Bricks => 0x2D,
            Block::TNT => 0x2E,
            Block::Bookshelf => 0x2F,
            Block::MossyCobblestone => 0x30,
            Block::Obsidian => 0x31
        }
    }
}

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
    x: usize,
    /// Height of the map
    y: usize,
    /// Length of the map
    z: usize,
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
    blocks: Vec<u8>,
    // metadata: Vec<Metadata>
}

impl ClassicWorld {
    //! Create a new empty world with a name aswell as dimensions
    pub fn new(name: &str, x: usize, y: usize, z: usize) -> Self {

        let mut blocks: Vec<u8> =
            (0..(x * y * z)).map(|k| Block::Air.into()).collect();
        for i in 0..x * z * 6 {
            blocks[i] = Block::GrassBlock.into();
        }
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
                y,
                z: 0,
                h: 0,
                p: 0
            },
            blocks
        }
    }

    pub fn get_size(&self) -> [usize; 3] {
        [self.x, self.y, self.z]
    }

    pub fn get_blocks(&self) -> &Vec<u8> {
        &self.blocks
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        let pos = x + (self.x as usize * z) + ((self.z as usize * self.x as usize)  * y);
        if y < self.y {
            self.blocks[pos as usize] = block.into();
        }
    }
    pub fn get_block(&mut self, x: usize, y: usize, z: usize) -> Block {
        let pos = x + (self.x as usize * z) + ((self.z as usize * self.x as usize)  * y);
        self.blocks[pos as usize].into()
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
    x: usize,
    y: usize,
    z: usize,
    // Heading
    h: u8,
    // Pitch
    p: u8,
}