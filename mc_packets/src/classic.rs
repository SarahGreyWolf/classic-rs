//! # Classic
//! The packets both ClientBound and ServerBound used for classic minecraft protocol 7

use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor};
use std::convert::TryInto;
use crate::Packet;

type Short = i16;
type ByteArray = [u8; 1024];

/// Packets to be sent to the clients
pub enum ClientBound {
    ServerIdentification(u8, String, String, u8),
    Ping,
    LevelInitialize,
    LevelDataChunk(Short, ByteArray, u8),
    LevelFinalize(Short, Short, Short),
    SetBlock(Short, Short, Short, u8),
    SpawnPlayer(i8, String, Short, Short, Short, u8, u8),
    PlayerTeleport(i8, Short, Short, Short, u8, u8),
    PositionAndOrientationUpdate(i8, i8, i8, i8, u8, u8),
    PositionUpdate(i8, i8, i8, i8),
    OrientationUpdate(i8, u8, u8),
    DespawnPlayer(i8),
    Message(i8, String),
    DisconnectPlayer(String),
    UpdateUserType(u8),
}

impl Packet<&[u8]> for ClientBound {
    fn from(buffer: &[u8]) -> Self {
        unimplemented!()
    }

    fn into(self) -> Vec<u8> {
        match self {
            ClientBound::ServerIdentification(prot_v, server_name, server_motd, u_type) => {
                let mut s_identification: Vec<u8> = vec![0x00];
                s_identification.push(prot_v);
                for x in server_name.into_bytes() {
                    s_identification.push(x);
                }
                for x in server_motd.into_bytes() {
                    s_identification.push(x);
                }
                s_identification.push(u_type);
                s_identification
            },
            ClientBound::Ping => {
                vec![0x01]
            },
            ClientBound::LevelInitialize => {
                vec![0x02]
            },
            ClientBound::LevelDataChunk(chunk_length, chunk_data, p_complete) => {
                let mut level_data_chunk: Vec<u8> = vec![0x03];
                level_data_chunk.push((chunk_length >> 8) as u8);
                level_data_chunk.push(chunk_length as u8);
                for x in chunk_data.to_vec() {
                    level_data_chunk.push(x);
                }
                level_data_chunk.push(p_complete);
                level_data_chunk
            },
            ClientBound::LevelFinalize(width, height, depth) => {
                let mut level_finalize: Vec<u8> = vec![0x04];
                level_finalize.push((width >> 8) as u8);
                level_finalize.push(width as u8);
                level_finalize.push((height >> 8) as u8);
                level_finalize.push(height as u8);
                level_finalize.push((depth >> 8) as u8);
                level_finalize.push(depth as u8);
                level_finalize
            },
            ClientBound::SetBlock(x, y, z, block) => {
                let mut set_block: Vec<u8> = vec![0x06];
                set_block.push((x >> 8) as u8);
                set_block.push(x as u8);
                set_block.push((y >> 8) as u8);
                set_block.push(y as u8);
                set_block.push((z >> 8) as u8);
                set_block.push(z as u8);
                set_block.push(block);
                set_block
            },
            ClientBound::SpawnPlayer(
                origin_p_id, origin_p_name, x, y, z, yaw, pitch) => {
                let mut spawn_player: Vec<u8> = vec![0x07];
                spawn_player.push(origin_p_id.try_into().unwrap());
                for x in origin_p_name.into_bytes() {
                    spawn_player.push(x)
                }
                spawn_player.push((x >> 8) as u8);
                spawn_player.push(x as u8);
                spawn_player.push((y >> 8) as u8);
                spawn_player.push(y as u8);
                spawn_player.push((z >> 8) as u8);
                spawn_player.push(z as u8);
                spawn_player.push(yaw);
                spawn_player.push(pitch);
                spawn_player
            },
            ClientBound::PlayerTeleport(origin_p_id, x, y, z, yaw, pitch) => {
                let mut player_teleport: Vec<u8> = vec![0x08];
                player_teleport.push(origin_p_id.try_into().unwrap());
                player_teleport.push((x >> 8) as u8);
                player_teleport.push(x as u8);
                player_teleport.push((y >> 8) as u8);
                player_teleport.push(y as u8);
                player_teleport.push((z >> 8) as u8);
                player_teleport.push(z as u8);
                player_teleport.push(yaw);
                player_teleport.push(pitch);
                player_teleport
            },
            ClientBound::PositionAndOrientationUpdate(
                origin_p_id, x_change, y_change, z_change, yaw, pitch)=> {
                let mut pos_orient_update: Vec<u8> = vec![0x09];
                pos_orient_update.push(origin_p_id.try_into().unwrap());
                pos_orient_update.push(x_change.try_into().unwrap());
                pos_orient_update.push(y_change.try_into().unwrap());
                pos_orient_update.push(z_change.try_into().unwrap());
                pos_orient_update.push(yaw);
                pos_orient_update.push(pitch);
                pos_orient_update
            },
            ClientBound::PositionUpdate(origin_p_id, x_change, y_change, z_change) => {
                let mut pos_update: Vec<u8> = vec![0x0A];
                pos_update.push(origin_p_id.try_into().unwrap());
                pos_update.push(x_change.try_into().unwrap());
                pos_update.push(y_change.try_into().unwrap());
                pos_update.push(z_change.try_into().unwrap());
                pos_update
            },
            ClientBound::OrientationUpdate(origin_p_id, yaw, pitch) => {
                let mut orient_update: Vec<u8> = vec![0x0B];
                orient_update.push(origin_p_id.try_into().unwrap());
                orient_update.push(yaw);
                orient_update.push(pitch);
                orient_update
            },
            ClientBound::DespawnPlayer(origin_p_id) => {
                vec![0x0C, origin_p_id.try_into().unwrap()]
            },
            ClientBound::Message(origin_p_id, msg) => {
                let mut message: Vec<u8> = vec![0x0D];
                message.push(origin_p_id.try_into().unwrap());
                for x in msg.into_bytes(){
                    message.push(x);
                }
                message
            },
            ClientBound::DisconnectPlayer(reason) => {
                let mut disconnect_player: Vec<u8> = vec![0x0E];
                for x in reason.into_bytes() {
                    disconnect_player.push(x);
                }
                disconnect_player
            },
            ClientBound::UpdateUserType(u_type) => {
                vec![0x0F, u_type]
            }
        }
    }
}

/// Packets to be sent to servers
pub enum ServerBound {
    // Final Byte unused, always 0x00
    // TODO: Implement proper identification
    // PlayerIdentification(u8, String, String, u8),
    PlayerIdentification(u8, String),
    SetBlock(Short, Short, Short, u8, u8),
    PositionAndOrientation(u8, Short, Short, Short, u8, u8),
    // Byte Unused, always 0xFF
    Message(u8, String),
    UnknownPacket
}

impl Packet<&[u8]> for ServerBound {
    fn from(buffer: &[u8]) -> Self {
        let mut cursor: Cursor<&[u8]> = Cursor::new(buffer);
        let id = cursor.read_u8().unwrap();
        match id {
            0x00 => {
                let protocol = cursor.read_u8().unwrap();
                let msg = buffer[cursor.position() as usize..].to_vec().into_iter()
                    .take_while(|&x| x != (0 as u8)).collect::<Vec<_>>();
                let msg = String::from_utf8(msg)
                    .expect("Invalid utf8 Message").replace("\u{20}", "");
                ServerBound::PlayerIdentification(protocol, msg)
            }
            0x05 => {
                let x: Short = cursor.read_i16::<BigEndian>().unwrap();
                let y: Short = cursor.read_i16::<BigEndian>().unwrap();
                let z: Short = cursor.read_i16::<BigEndian>().unwrap();
                let mode: u8 = cursor.read_u8().unwrap();
                let b_type: u8 = cursor.read_u8().unwrap();
                ServerBound::SetBlock(x, y, z, mode, b_type)
            }
            0x08 => {
                let player_id: u8 = cursor.read_u8().unwrap();
                let x: Short = cursor.read_i16::<BigEndian>().unwrap();
                let y: Short = cursor.read_i16::<BigEndian>().unwrap();
                let z: Short = cursor.read_i16::<BigEndian>().unwrap();
                let yaw: u8 = cursor.read_u8().unwrap();
                let pitch: u8 = cursor.read_u8().unwrap();
                ServerBound::PositionAndOrientation(player_id, x, y, z, yaw, pitch)
            }
            0x0d => {
                let unused: u8 = cursor.read_u8().unwrap();
                let msg = buffer[cursor.position() as usize..].to_vec().into_iter()
                    .take_while(|&x| x != (0 as u8)).collect::<Vec<_>>();
                let msg = String::from_utf8(msg)
                    .expect("Invalid utf8 Message").replace("\u{20}", "");
                ServerBound::Message(unused, msg)
            }
            _ => {
                ServerBound::UnknownPacket
            }
        }
    }

    fn into(self) -> Vec<u8> {
        unimplemented!()
    }
}