use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt, Error, ErrorKind};
use tokio::sync::{Mutex, MutexGuard};
use flume::{Receiver, Sender};
use log::{info, debug, error, warn};
use std::sync::{Arc};
use std::ops::{Deref, DerefMut};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::cell::{RefMut};
use md5::{Md5, Digest};




use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};
use mc_worlds::classic::{ClassicWorld, Block};

use crate::config::Config;
use md5::digest::Update;

const STRING_LENGTH: usize = 64;

pub struct Client {
    pub(crate) username: String,
    ip: String,
    id: u8,
    // The rank of the user, 0x64 for op, 0x00 for normal
    user_type: u8,
    logged_in: bool,
    socket: TcpStream,
    n_tx: Sender<(u8, Vec<ClientBound>)>,
    current_x: i16,
    current_y: i16,
    current_z: i16,
    current_yaw: u8,
    current_pitch: u8,
    flying: bool,
}

impl Client {
    pub async fn new(id: u8, sock: TcpStream, n_tx: Sender<(u8, Vec<ClientBound>)>) -> Self {
        Self {
            username: "".to_string(),
            ip: sock.peer_addr().expect("Failed to get peers address").ip().to_string(),
            id,
            user_type: 0x00,
            logged_in: false,
            socket: sock,
            n_tx,
            current_x: 0,
            current_y: 0,
            current_z: 0,
            current_yaw: 0,
            current_pitch: 0,
            flying: false,
        }
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub fn get_ip(&self) -> String {
        self.ip.clone()
    }

    pub async fn spawn_self(&self) -> ClientBound {
        ClientBound::SpawnPlayer(
            self.id,
            self.get_username_as_bytes(),
            self.current_x,
            self.current_y,
            self.current_z,
            self.current_yaw,
            self.current_pitch
        )
    }

    pub fn despawn_self(&self) -> [ClientBound; 2] {
        info!("{} has left the server", self.username);
        [
            ClientBound::DespawnPlayer(self.id),
            ClientBound::Message(255, {
                let msg = format!("{} left the Server", self.username);
                encode_string(&msg)
            })
        ]
    }

    pub async fn handle_connect(&mut self, salt: &str, world: Arc<Mutex<ClassicWorld>>) -> Result<(), tokio::io::Error> {
        let mut receive_buffer = [0x00; 1460];
        self.socket.read(&mut receive_buffer).await?;

        let mut serverbound_packets: Vec<ServerBound> = Vec::new();
        let mut clientbound_packets: Vec<ClientBound> = Vec::new();
        let mut echo_packets: Vec<ClientBound> = Vec::new();

        let mut buffer_handled: usize = 0;
        while buffer_handled < receive_buffer[..].len() &&
            buffer_handled + ServerBound::size(*&receive_buffer[buffer_handled]) < receive_buffer.len() {
            if self.logged_in && *&receive_buffer[buffer_handled] == 0x00  {
                break;
            }
            if receive_buffer[buffer_handled..buffer_handled +
                    ServerBound::size(*&receive_buffer[buffer_handled])].len() == 0 {
                break;
            }
            serverbound_packets.push(
                Packet::from(
                    &receive_buffer[buffer_handled..buffer_handled +
                            ServerBound::size(*&receive_buffer[buffer_handled])])
            );
            if !self.logged_in {
                self.logged_in = *&receive_buffer[buffer_handled] == 0x00;
            }
            buffer_handled += ServerBound::size(*&receive_buffer[buffer_handled]);
        }

        for packet in serverbound_packets {
            match packet {
                ServerBound::PlayerIdentification(protocol, username,
                                                  key, _) => {
                    if username.is_empty() {
                        self.socket.shutdown(std::net::Shutdown::Both).expect("Failed to shutdown socket");
                        break;
                    }
                    if protocol != 0 {
                        let mut world_lock = world.lock().await;
                        self.username = username;
                        if self.username == "" {break}
                        let config = Config::get();
                        if config.server.online_mode {
                            let mut hasher = Md5::new();
                            let mut concat: Vec<u8> = vec![];
                            concat.append(&mut salt.as_bytes().to_vec());
                            concat.append(&mut self.username.as_bytes().to_vec());
                            md5::digest::Update::update(&mut hasher, concat.as_slice());
                            let hash = hasher.finalize().to_vec();
                            let hash_string: String = hash.iter().map(|e| format!("{:02x}", e)).collect();
                            // debug!("{:x?}:{:x?}", key, hash_string);
                            if key != hash_string {
                                self.disconnect("You are not logged in to Minecraft").await?;
                                break;
                            }
                        }
                        self.write_packets(vec![ClientBound::ServerIdentification(
                            7,
                            encode_string(&config.server.name),
                            encode_string(&config.server.motd),
                            0x00,
                        ), ClientBound::LevelInitialize]).await;
                        self.send_blocks(world_lock.deref_mut()).await.expect("Failed to send blocks");
                        let size = world_lock.get_size();
                        self.write_packets(vec![
                            ClientBound::LevelFinalize(size[0], size[1], size[2]),
                            ClientBound::PlayerTeleport(
                                255,
                                (((size[0] / 2) * 32) + 16) as i16,
                                (((size[1] / 2) + 2) * 32) as i16,
                                (((size[2] / 2) * 32) + 16) as i16,
                                0,
                                0,
                            )
                        ]).await;
                        self.current_x = (((size[0] / 2) * 32) + 16) as i16;
                        self.current_y = (((size[1] / 2) + 2) * 32) as i16;
                        self.current_z = (((size[2] / 2) * 32) + 16) as i16;
                        echo_packets.push(ClientBound::SpawnPlayer(
                            255,
                            self.get_username_as_bytes(),
                            self.current_x,
                            self.current_y,
                            self.current_z,
                            0,
                            0,
                        ));
                        info!("{} joined the Server", self.username);
                        clientbound_packets.push(ClientBound::Message(255, {
                                let msg = format!("{} joined the Server", self.username);
                                encode_string(&msg)
                            }
                        ));
                        clientbound_packets.push(ClientBound::SpawnPlayer(
                            self.id,
                            self.get_username_as_bytes(),
                            self.current_x,
                            self.current_y,
                            self.current_z,
                            self.current_yaw,
                            self.current_pitch,
                        ));
                        drop(world_lock);
                    }
                }
                ServerBound::PositionAndOrientation(
                    p_id, x, y, z, yaw, pitch) => {
                    let mut pos_changed: bool = false;
                    let mut ori_changed: bool = false;
                    let y = y + 3;
                    let moved_teleport_distance =
                        -15 < (self.current_x - x) && (self.current_x - x) > 15 ||
                        -15 < (self.current_y - y) && (self.current_y - y) > 15 ||
                        -15 < (self.current_z - z) && (self.current_z - z) > 15;
                    if x != self.current_x || y != self.current_y || z != self.current_z {
                        pos_changed = true;
                        // debug!("{:#}:{:#}:{:#}", self.current_x - x, self.current_y - y, self.current_z - z);
                    }
                    if yaw != self.current_yaw || pitch != self.current_pitch {
                        ori_changed = true;
                    }
                    if pos_changed && ori_changed {
                        if  moved_teleport_distance {
                            clientbound_packets.push(
                                ClientBound::PositionAndOrientationUpdate(
                                    self.id,
                                    -(self.current_x - x) as i8,
                                    -(self.current_y - y) as i8,
                                    -(self.current_z - z) as i8,
                                    yaw,
                                    pitch
                                )
                            );
                        } else {
                            clientbound_packets.push(
                                ClientBound::PlayerTeleport(
                                    self.id,
                                    x,
                                    y,
                                    z,
                                    yaw,
                                    pitch
                                )
                            )
                        }
                    } else if pos_changed {
                        if  moved_teleport_distance {
                            clientbound_packets.push(
                                ClientBound::PositionUpdate(
                                    self.id,
                                    -(self.current_x - x) as i8,
                                    -(self.current_y - y) as i8,
                                    -(self.current_z - z) as i8,
                                )
                            );
                        } else {
                            clientbound_packets.push(
                                ClientBound::PlayerTeleport(
                                    self.id,
                                    x,
                                    y,
                                    z,
                                    yaw,
                                    pitch
                                )
                            )
                        }

                    } else if ori_changed {
                        clientbound_packets.push(
                            ClientBound::OrientationUpdate(
                                self.id,
                                yaw,
                                pitch
                            )
                        );
                    } else {}

                    self.current_x = x;
                    self.current_y = y;
                    self.current_z = z;
                    self.current_yaw = yaw;
                    self.current_pitch = pitch;
                }
                ServerBound::SetBlock(x, y, z, mode, block) => {
                    let mut world_lock = world.lock().await;
                    let block = Block::from(block).clone();
                    if mode == 0x00 {
                        if block != Block::Bedrock {
                            world_lock.set_block(x as usize, y as usize, z as usize, Block::Air.into());
                            echo_packets.push(
                                ClientBound::SetBlock(x, y, z, Block::Air.into())
                            );
                            clientbound_packets.push(
                                ClientBound::SetBlock(x, y, z, Block::Air.into())
                            );
                        } else {
                            echo_packets.push(
                                ClientBound::SetBlock(x, y, z, Block::Bedrock.into())
                            );
                        }
                    } else {
                        let (n_x, n_y, n_z, block) =
                            world_lock.set_block(x as usize, y as usize, z as usize, block.into());
                        echo_packets.push(
                            ClientBound::SetBlock(x, y, z, Block::Air.into())
                        );
                        echo_packets.push(
                            ClientBound::SetBlock(n_x as i16, n_y as i16, n_z as i16, block.into())
                        );
                        clientbound_packets.push(
                            ClientBound::SetBlock(n_x as i16, n_y as i16, n_z as i16, block.into())
                        );
                    }
                    drop(world_lock);
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
                    let mut msg = self.build_message(
                        self.username.as_str(), self.id, f_msg.as_str()).await;
                    echo_packets.append(&mut msg.clone());
                    clientbound_packets.append(&mut msg.clone());
                }
                ServerBound::UnknownPacket => {
                    let msg = String::from_utf8(receive_buffer.to_vec())
                        .expect("Invalid utf8 Message");
                    debug!("{}: {}", self.username, msg);
                }
            }
        }
        echo_packets.push(ClientBound::Ping);

        self.write_packets(echo_packets).await;
        self.n_tx.send((self.id, clientbound_packets)).expect("Failed to send Packets");
        Ok(())
    }

    pub async fn disconnect(&mut self, msg: &str) -> Result<(), tokio::io::Error> {
        self.write_packets(vec![ClientBound::DisconnectPlayer({
            encode_string(msg)
        })]).await;

        Ok(())
    }

    pub async fn send_message(&mut self, messages: Vec<ClientBound>) {
        self.write_packets(messages).await;
    }

    pub async fn build_message(&self, sender_name: &str, sender_id: u8, msg: &str) -> Vec<ClientBound> {
        let mut messages: Vec<ClientBound> = vec![];
        let split_msg = msg.split_ascii_whitespace();
        if split_msg.clone().count() > 1 {
            let mut split_indx = 0;
            let mut char_count: usize = 0;
            let msg_full: String = split_msg.map(|word|
                if char_count + word.len() < (64 - (sender_name.len() + 4)) {
                    split_indx += 1;
                    char_count += word.len() + 1;
                    format!("{} ", word)
                } else { "".to_string() }
            ).collect();
            let first_msg = format!("<{}>: {}", sender_name, msg_full);
            info!("{}", first_msg);
            messages.push(ClientBound::Message(sender_id, encode_string(&first_msg)));
            while char_count < msg.len() {
                let mut fresh_char_count = 0;
                let split_msg = msg.split_at(char_count).1.split_ascii_whitespace();
                let msg_full: String = split_msg.map(|word|
                    if fresh_char_count + word.len() < 64 {
                        fresh_char_count += word.len() + 1;
                        char_count += word.len() + 1;
                        format!("{} ", word)
                    } else { "".to_string() }
                ).collect();
                let msg = format!("{}", msg_full);
                info!("<{}>: {}", sender_name, msg_full);
                messages.push(ClientBound::Message(sender_id, encode_string(&msg)));
            }
        }else {
            let first_msg = format!("<{}>: {}", sender_name, msg);
            info!("{}", first_msg);
            messages.push(ClientBound::Message(sender_id, encode_string(&first_msg)));
        }
        messages
    }

    async fn send_blocks(&mut self, world: &mut ClassicWorld) -> Result<(), tokio::io::Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write(&(world.get_blocks().len() as u32).to_be_bytes()).unwrap();
        encoder.write_all(world.get_blocks().as_slice()).unwrap();
        let compressed = encoder.finish().expect("Failed to compress data");
        // let compressed = world.get_gzipped();
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
                self.write_packets(vec![world_packet]).await;
            }else {
                // let world_packet = ClientBound::LevelDataChunk(left as i16, send_buffer, progress);
                sent += left;
                left = 0;
                let world_packet = ClientBound::LevelDataChunk(1024, send_buffer,
                                                               ((sent / compressed.len()) * 100) as u8);
                self.write_packets(vec![world_packet]).await;
            }
        }

        Ok(())
    }

    pub async fn write_packets(&mut self, packets: Vec<ClientBound>) {
        let mut packet_buffer: [u8; 1460] = [0u8; 1460];
        let mut buffer_filled: usize = 0;
        for p in 0..packets.len() {
            let c_packet = Packet::into(&packets[p]);
            let c_slice = c_packet.as_slice();
            if c_slice.len() > (1460 - buffer_filled) {
                match self.socket.write_all(&packet_buffer[0..buffer_filled]).await {
                    Ok(_) => {}
                    Err(e) => {
                        if e.kind() == ErrorKind::ConnectionAborted || e.kind() == ErrorKind::ConnectionReset {
                            break;
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
                } else { break; }
            }
            buffer_filled += c_slice.len();
        }

        match self.socket.write_all(&packet_buffer[0..buffer_filled]).await {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == ErrorKind::ConnectionAborted || e.kind() == ErrorKind::ConnectionReset {} else {
                    panic!("Error: {:?}", e);
                }
            }
        }
    }

    fn get_username_as_bytes(&self) -> [u8; 64] {
        let mut username: [u8; 64] = [0x20; 64];
        for i in 0..username.len() {
            if i < self.username.len() {
                username[i] = self.username.as_bytes()[i];
            }
        }
        username
    }

}

fn encode_string(string: &str) -> [u8; 64] {
    let mut string_bytes: [u8; 64] = [0x20; 64];
    for i in 0..string.len() {
        if i >= string_bytes.len() {
            break;
        }
        string_bytes[i] = string.as_bytes()[i];
    }
    string_bytes
}