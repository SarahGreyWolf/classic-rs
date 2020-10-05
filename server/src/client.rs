use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt, Error, ErrorKind};
use flume::{Receiver, Sender};
use log::{info, debug, error, warn};
use std::sync::{Arc, Mutex, MutexGuard};
use std::ops::Deref;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::cell::{RefMut};


use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};
use mc_worlds::classic::{ClassicWorld, Block};

use crate::config::Config;
use std::borrow::BorrowMut;

const STRING_LENGTH: usize = 64;

pub struct Client {
    pub(crate) username: String,
    id: u8,
    // The rank of the user, 0x64 for op, 0x00 for normal
    user_type: u8,
    socket: TcpStream,
    n_tx: Sender<(u8, Vec<ClientBound>)>,
    current_x: i16,
    current_y: i16,
    current_z: i16,
    current_yaw: u8,
    current_pitch: u8,
}

impl Client {
    pub async fn new(id: u8, sock: TcpStream, n_tx: Sender<(u8, Vec<ClientBound>)>) -> Self {
        Self {
            username: "".to_string(),
            id,
            user_type: 0x00,
            socket: sock,
            n_tx,
            current_x: 0,
            current_y: 0,
            current_z: 0,
            current_yaw: 0,
            current_pitch: 0,
        }
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub async fn spawn_self(&self) -> ClientBound {
        ClientBound::SpawnPlayer(
            self.id,
            self.get_username_as_bytes(),
            self.current_x,
            self.current_y,
            self.current_y,
            self.current_yaw,
            self.current_pitch
        )
    }

    pub async fn handle_connect(&mut self, world: Arc<Mutex<ClassicWorld>>) -> Result<(), tokio::io::Error> {
        let mut world_lock = world.try_lock().unwrap();
        let mut receive_buffer = [0 as u8; 1460];
        let mut send_index: u8 = 0;
        self.socket.read(&mut receive_buffer).await?;

        let mut serverbound_packets: Vec<ServerBound> = Vec::new();
        let mut clientbound_packets: Vec<ClientBound> = Vec::new();
        let mut echo_packets: Vec<ClientBound> = Vec::new();

        match receive_buffer[0] {
            0x08 => {
                serverbound_packets.push(Packet::from(&receive_buffer[0..10]));
                serverbound_packets.push(Packet::from(&receive_buffer[10..]));
            }
            _ => {
                serverbound_packets.push(Packet::from(receive_buffer.as_ref()));
            }
        }

        for packet in serverbound_packets {
            match packet {
                ServerBound::PlayerIdentification(protocol, username,
                                                  ver_key, _) => {
                    if protocol != 0 {
                        self.username = username;
                        // debug!("{}", ver_key);
                        let config = Config::get();
                        let mut name: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
                        for i in 0..config.server.name.len() {
                            name[i] = config.server.name.as_bytes()[i];
                        }
                        let mut motd: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
                        for i in 0..config.server.motd.len() {
                            motd[i] = config.server.motd.as_bytes()[i];
                        }
                        self.write_packets(&vec![ClientBound::ServerIdentification(
                            7,
                            name,
                            motd,
                            0x00,
                        ), ClientBound::LevelInitialize]).await;
                        self.send_blocks(world_lock.deref()).await.expect("Failed to send blocks");
                        let size = world_lock.get_size();
                        self.write_packets(&vec![
                            ClientBound::LevelFinalize(size[0], size[1], size[2]),
                            ClientBound::SpawnPlayer(
                                255,
                                self.get_username_as_bytes(),
                                5 * 32,
                                7 * 32,
                                5 * 32,
                                0,
                                0,
                            ),
                            ClientBound::PlayerTeleport(
                                255,
                                5 * 32,
                                7 * 32,
                                5 * 32,
                                0,
                                0,
                            )
                        ]).await;
                        info!("{} joined the Server", self.username);
                        clientbound_packets.push(ClientBound::Message(255, {
                                let msg = format!("{} joined the Server", self.username);
                                let mut message: [u8; 64] = [0x20; 64];
                                for i in 0..msg.len() {
                                    message[i] = msg.as_bytes()[i];
                                }
                                message
                            }
                        ));
                        clientbound_packets.push(ClientBound::SpawnPlayer(
                            self.id,
                            self.get_username_as_bytes(),
                            5 * 32,
                            7 * 32,
                            5 * 32,
                            0,
                            0,
                        ));
                    }
                }
                ServerBound::SetBlock(x, y, z, mode, block) => {
                    let block = Block::from(block).clone();
                    if mode == 0x00 {
                        world_lock.set_block(x, y, z, Block::Air.into());
                        echo_packets.push(
                            ClientBound::SetBlock(x, y, z, Block::Air.into())
                        );
                        clientbound_packets.push(
                            ClientBound::SetBlock(x, y, z, Block::Air.into())
                        );
                    } else {
                        world_lock.set_block(x, y, z, block.into());
                        echo_packets.push(
                            ClientBound::SetBlock(x, y, z, block.into())
                        );
                        clientbound_packets.push(
                            ClientBound::SetBlock(x, y, z, block.into())
                        );
                    }
                }
                ServerBound::PositionAndOrientation(
                    p_id, x, y, z, yaw, pitch) => {
                    let mut pos_changed: bool = false;
                    let mut ori_changed: bool = false;
                    if x != self.current_x || y != self.current_y || z != self.current_z {
                        pos_changed = true;
                    }
                    if yaw != self.current_yaw || pitch != self.current_pitch {
                        ori_changed = true;
                    }
                    if pos_changed && ori_changed {
                        echo_packets.push(
                            ClientBound::PositionAndOrientationUpdate(
                                255,
                                (self.current_x - x) as i8,
                                (self.current_y - y) as i8,
                                (self.current_z - z) as i8,
                                yaw,
                                pitch
                            )
                        );
                        clientbound_packets.push(
                            ClientBound::PositionAndOrientationUpdate(
                                p_id,
                                (self.current_x - x) as i8,
                                (self.current_y - y) as i8,
                                (self.current_z - z) as i8,
                                yaw,
                                pitch
                            )
                        );
                    } else if pos_changed {
                        echo_packets.push(
                            ClientBound::PositionUpdate(
                                255,
                                (self.current_x - x) as i8,
                                (self.current_y - y) as i8,
                                (self.current_z - z) as i8,
                            )
                        );
                        clientbound_packets.push(
                            ClientBound::PositionUpdate(
                                p_id,
                                (self.current_x - x) as i8,
                                (self.current_y - y) as i8,
                                (self.current_z - z) as i8,
                            )
                        );
                    } else {
                        echo_packets.push(
                            ClientBound::OrientationUpdate(
                                255,
                                yaw,
                                pitch
                            )
                        );
                        clientbound_packets.push(
                            ClientBound::OrientationUpdate(
                                p_id,
                                yaw,
                                pitch
                            )
                        );
                    }

                    self.current_x = x;
                    self.current_y = y;
                    self.current_z = z;
                    self.current_yaw = yaw;
                    self.current_pitch = pitch;
                }
                ServerBound::Message(_, message) => {
                    let mut f_msg: String = "".to_owned();
                    let split: Vec<&str> = message.split_ascii_whitespace().collect();
                    for i in 0..split.len() {
                        if i != 0 {
                            f_msg.push_str(&format!(" {}", split[i]));
                        }else{
                            f_msg.push_str(&format!("{}", split[i]));
                        }
                    }

                    let msg = format!("<{}>: {}", self.username, f_msg);
                    info!("{}", msg);
                    let mut message: [u8; 64] = [0x20; 64];
                    for i in 0..msg.len() {
                        message[i] = msg.as_bytes()[i];
                    }
                    echo_packets.push(ClientBound::Message(self.id, message));
                    clientbound_packets.push(ClientBound::Message(self.id, message));
                }
                ServerBound::UnknownPacket => {
                    let msg = String::from_utf8(receive_buffer.to_vec())
                        .expect("Invalid utf8 Message");
                    debug!("{}: {}", self.username, msg);
                }
            }
        }
        echo_packets.push(ClientBound::Ping);

        self.write_packets(&vec![ClientBound::Ping]).await;

        self.write_packets(&echo_packets).await;
        self.n_tx.send((self.id, clientbound_packets)).expect("Failed to send Packets");
        Ok(())
    }

    pub async fn disconnect(&mut self, msg: &str) -> Result<(), tokio::io::Error> {
        self.write_packets(&vec![ClientBound::DisconnectPlayer({
            let mut message_bytes: [u8; 64] = [0x20; 64];
            for i in 0..msg.len() {
                message_bytes[i] = msg.as_bytes()[i];
            }
            message_bytes
        })]).await;

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
                self.write_packets(&vec![world_packet]).await;
            }else {
                // let world_packet = ClientBound::LevelDataChunk(left as i16, send_buffer, progress);
                sent += left;
                left = 0;
                let world_packet = ClientBound::LevelDataChunk(1024, send_buffer,
                                                               ((sent / compressed.len()) * 100) as u8);
                self.write_packets(&vec![world_packet]).await;
            }
        }

        Ok(())
    }

    pub async fn write_packets(&mut self, packets: &Vec<ClientBound>) {
        let mut packet_buffer: [u8; 1460] = [0u8; 1460];
        let mut buffer_filled: usize = 0;

        for p in 0..packets.len() {
            let c_packet = Packet::into(&packets[p]);
            let c_slice = c_packet.as_slice();
            if c_slice.len() > (1460 - buffer_filled) {
                match self.socket.write_all(&packet_buffer[0..buffer_filled]).await {
                    Ok(_) => {}
                    Err(e) => {
                        if e.kind() == ErrorKind::ConnectionAborted {
                            // debug!("Fucked Before");
                        } else {
                            panic!("Error: {:?}", e);
                        }
                    }
                }
                buffer_filled = 0;
                packet_buffer = [0u8; 1460];
            }
            let mut s_index = 0;
            for i in buffer_filled..1460 {
                if s_index < c_slice.len() {
                    packet_buffer[i] = c_slice[s_index];
                    s_index += 1;
                } else {break;}
            }
            buffer_filled += c_slice.len();
        }

        match self.socket.write_all(&packet_buffer[0..buffer_filled]).await {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == ErrorKind::ConnectionAborted {
                    // debug!("Fucked Before");
                } else {
                    panic!("Error: {:?}", e);
                }
            }
        }
        // debug!("{:x?}, {:x?}", packets, packet_buffer);
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
