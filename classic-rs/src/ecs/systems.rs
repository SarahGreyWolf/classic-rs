use specs::{System, WriteStorage, ReadStorage, Join};
use log::{debug};

use crate::ecs::components::player::{Stream, Player};
use crate::ecs::components::entity::{Rotation, DeltaVel};
use crate::ecs::components::common::Pos;
use mc_packets::Packet;
use mc_packets::classic::{ServerBound, ClientBound};
use crate::config::Config;
use std::io::Write;

const STRING_LENGTH: usize = 64;

pub struct NetworkReadSys;

// impl<'a> System<'a> for NetworkReadSys {
//     type SystemData = (WriteStorage<'a, Stream>, WriteStorage<'a, Pos>, WriteStorage<'a, Rotation>,
//                        WriteStorage<'a, DeltaVel>, WriteStorage<'a, Player>);
//
//     fn run(&mut self, (mut stream, mut pos, mut rot, mut delta, mut player): Self::SystemData) {
//         for (stream, pos, rot, delta, player)
//             in (&mut stream, &mut pos, &mut rot, &mut delta, &mut player).join() {
//             debug!("Doing a thing");
//             let mut buffer = [0 as u8; 1460];
//             stream.0.peek(&mut buffer).expect("Failed to peek at incoming");
//
//             let incoming_packet = Packet::from(buffer.as_ref());
//             match incoming_packet {
//                 ServerBound::PlayerIdentification(_protocol, username,
//                                                   ver_key, _) => {
//                     player.username = username;
//                     player.ver_key = ver_key;
//                     debug!("{}", player.username);
//                     debug!("{}", player.ver_key);
//                     let config = Config::get();
//                     let mut name: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
//                     for i in 0..config.server.name.len() {
//                         name[i] = config.server.name.as_bytes()[i];
//                     }
//                     let mut motd: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
//                     for i in 0..config.server.motd.len() {
//                         motd[i] = config.server.motd.as_bytes()[i];
//                     }
//                     let data = Packet::into(
//                         ClientBound::ServerIdentification(
//                             7,
//                             name,
//                             motd,
//                             0x00
//                         )
//                     );
//                     stream.0.write(data.as_slice()).expect("Failed to write data");
//                     stream.0.write(Packet::into(ClientBound::LevelInitialize).as_slice()).
//                         expect("Failed to write data");
//                 }
//                 ServerBound::SetBlock(_, _, _, _, _) => {}
//                 ServerBound::PositionAndOrientation(
//                     p_id, x, y, z, yaw, pitch) => {
//                     pos.x = x;
//                     pos.y = y;
//                     pos.z = z;
//                     rot.yaw = yaw;
//                     rot.pitch = pitch;
//                 }
//                 ServerBound::Message(_, _) => {}
//                 ServerBound::UnknownPacket => {
//                     let msg = String::from_utf8(buffer.to_vec())
//                         .expect("Invalid utf8 Message");
//                     debug!("{}", msg);
//                 }
//             }
//         }
//     }
// }