use std::net::{TcpStream};
use std::io::Write;
use flume::{Receiver, Sender};
use log::{info, debug, error, warn};

use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};

use crate::config::Config;

const STRING_LENGTH: usize = 64;

pub struct Client {
    username: String,
    id: u8,
    // The rank of the user, 0x64 for op, 0x00 for normal
    user_type: u8,
    socket: Result<TcpStream, std::io::Error>,
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    current_x: i16,
    current_y: i16,
    current_z: i16
}

impl Client {
    pub fn new(id: u8, sock: Result<TcpStream, std::io::Error>, tx: Sender<Vec<u8>>,
                     rx: Receiver<Vec<u8>>) -> Self {
        Self {
            username: "".to_string(),
            id,
            user_type: 0x00,
            socket: sock,
            tx,
            rx,
            current_x: 0,
            current_y: 0,
            current_z: 0
        }
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }

    pub fn handle_connect(mut self) -> Result<(), std::io::Error> {
        match self.socket {
            Err(e) => eprintln!("Accept Failed = {:?}", e),
            Ok(mut sock) => {
                let mut buffer = [0 as u8; 1460];
                sock.peek(&mut buffer).expect("Failed to peek at incoming");

                let incoming_packet = Packet::from(buffer.as_ref());
                match incoming_packet {
                    ServerBound::PlayerIdentification(protocol, username,
                                                      ver_key, _) => {
                        self.username = username;
                        debug!("{}", self.username);
                        debug!("{}", ver_key);
                        let config = Config::get();
                        let mut name: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
                        for i in 0..config.name.len() {
                            name[i] = config.name.as_bytes()[i];
                        }
                        let mut motd: [u8; STRING_LENGTH] = [0x20; STRING_LENGTH];
                        for i in 0..config.motd.len() {
                            motd[i] = config.motd.as_bytes()[i];
                        }
                        let data = Packet::into(
                            ClientBound::ServerIdentification(
                                7,
                                name,
                                motd,
                                0x00
                            )
                        );
                        // sock.write(data.as_slice()).expect("Failed to write data");
                        sock.write_all(data.as_slice()).expect("Failed to write data");
                        sock.write(Packet::into(ClientBound::LevelInitialize).as_slice()).
                            expect("Failed to write data");
                    }
                    ServerBound::SetBlock(_, _, _, _, _) => {}
                    ServerBound::PositionAndOrientation(
                        p_id, x, y, z, yaw, pitch) => {
                        self.tx.send(
                            Packet::into(ServerBound::PositionAndOrientation(
                                self.id,
                                x, y, z, yaw, pitch
                            ))
                        ).expect("Failed to send PositionAndOrientation");
                    }
                    ServerBound::Message(_, _) => {}
                    ServerBound::UnknownPacket => {
                        let msg = String::from_utf8(buffer.to_vec())
                            .expect("Invalid utf8 Message");
                        debug!("{}", msg);
                    }
                }
                let received = self.rx.try_recv().unwrap_or(vec![]);
                debug!("{:x?}", received);
            }
        }
        Ok(())
    }
}