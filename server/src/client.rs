use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt, Error, ErrorKind};
use flume::{Receiver, Sender};
use log::{info, debug, error, warn};
use std::sync::{Arc, Mutex, MutexGuard};
use std::ops::Deref;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;


use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};
use mc_worlds::classic::{ClassicWorld, Block};

use crate::config::Config;

const STRING_LENGTH: usize = 64;

pub struct Client {
    pub(crate) username: String,
    id: i8,
    // The rank of the user, 0x64 for op, 0x00 for normal
    user_type: u8,
    socket: TcpStream,
    current_x: i16,
    current_y: i16,
    current_z: i16
}

impl Client {
    pub async fn new(id: i8, sock: TcpStream) -> Self {
        Self {
            username: "".to_string(),
            id,
            user_type: 0x00,
            socket: sock,
            current_x: 0,
            current_y: 0,
            current_z: 0
        }
    }

    pub fn get_id(&self) -> i8 {
        self.id
    }

    pub async fn handle_connect(&mut self, world: Arc<Mutex<ClassicWorld>>) -> Result<(), tokio::io::Error> {
        let mut world_lock = world.try_lock().unwrap();
        let mut buffer = [0 as u8; 1460];
        self.socket.read(&mut buffer).await?;

        let mut incoming_packets: Vec<ServerBound> = Vec::new();

        match buffer[0] {
            0x08 => {
                incoming_packets.push(Packet::from(&buffer[0..10]));
                incoming_packets.push(Packet::from(&buffer[10..]));
            }
            _ => {
                incoming_packets.push(Packet::from(buffer.as_ref()));
            }
        }

        for packet in incoming_packets {
            match packet {
                ServerBound::PlayerIdentification(protocol, username,
                                                  ver_key, _) => {
                    if protocol != 0 {
                        self.username = username;
                        debug!("{}", self.username);
                        debug!("{}", ver_key);
                        let config = Config::get();
                        let mut name: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
                        for i in 0..config.server.name.len() {
                            name[i] = config.server.name.as_bytes()[i];
                        }
                        let mut motd: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
                        for i in 0..config.server.motd.len() {
                            motd[i] = config.server.motd.as_bytes()[i];
                        }
                        self.socket_write(ClientBound::ServerIdentification(
                            7,
                            name,
                            motd,
                            0x00,
                        )).await;
                        self.socket_write(ClientBound::LevelInitialize).await;
                        self.send_blocks(world_lock.deref()).await.expect("Failed to send blocks");
                        let size = world_lock.get_size();
                        self.socket_write(ClientBound::LevelFinalize(size[0], size[1], size[2]))
                            .await;

                        self.socket_write(ClientBound::SpawnPlayer(
                            -1, self.get_username_as_bytes(), 0, 0, 0, 0, 0
                        )).await;
                    }
                }
                ServerBound::SetBlock(x, y, z, mode, block) => {
                    let block = Block::from(block);
                    if mode == 0x00 {
                        world_lock.set_block(x, y, z, Block::Air.into());
                    } else {
                        world_lock.set_block(x, y, z, block.into());
                    }
                }
                ServerBound::PositionAndOrientation(
                    p_id, x, y, z, yaw, pitch) => {
                    // self.tx.send(
                    //     Packet::into(ServerBound::PositionAndOrientation(
                    //         self.id,
                    //         x, y, z, yaw, pitch
                    //     ))
                    // ).expect("Failed to send PositionAndOrientation");
                }
                ServerBound::Message(_, message) => {
                    info!("{}: {}", self.username, message);
                }
                ServerBound::UnknownPacket => {
                    let msg = String::from_utf8(buffer.to_vec())
                        .expect("Invalid utf8 Message");
                    debug!("{}: {}", self.username, msg);
                }
            }
        }
        self.socket_write(ClientBound::Ping).await;
        // let received = self.rx.try_recv().unwrap_or(vec![]);
        // debug!("{:x?}", received);
        Ok(())
    }

    async fn send_blocks(&mut self, world: &ClassicWorld) -> Result<(), tokio::io::Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write(&(world.get_blocks().len() as u32).to_be_bytes()).unwrap();
        encoder.write_all(world.get_blocks()).unwrap();
        let compressed = encoder.finish().expect("Failed to compress data");
        let mut sent: usize = 0;
        let mut left: usize = compressed.len();

        while left > 0 && sent < compressed.len() {
            let mut send_buffer: [u8; 1024] = [0x00; 1024];
            for i in 0..1024 {
                if sent+i < compressed.len() {
                    send_buffer[i] = compressed[sent + i];
                }
            }
            if left > 1024 {
                let world_packet = ClientBound::LevelDataChunk(1024, send_buffer,
                                                               ((sent / compressed.len()) * 100) as u8);
                sent += 1024;
                left -= 1024;
                self.socket_write(world_packet).await;
            }else {
                // let world_packet = ClientBound::LevelDataChunk(left as i16, send_buffer, progress);
                sent += left;
                left = 0;
                let world_packet = ClientBound::LevelDataChunk(1024, send_buffer,
                                                               ((sent / compressed.len()) * 100) as u8);
                self.socket_write(world_packet).await;
            }
        }

        Ok(())
    }

    async fn socket_write(&mut self, packet: ClientBound) {
        match self.socket.write_all(Packet::into(packet).as_slice()).await {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == ErrorKind::ConnectionAborted {
                    info!("Player {} Lost Connection", self.username);
                } else {
                    panic!("Error: {:?}", e);
                }
            }
        }
    }

    fn get_username_as_bytes(&self) -> [u8; 64] {
        let mut username: [u8; 64] = [0u8; 64];
        for i in 0..username.len() {
            if i < self.username.len() {
                username[i] = self.username.as_bytes()[i];
            }
        }
        username
    }
}
