use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use flume::{Receiver, Sender};
use log::{info, debug, error, warn};

use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};

use crate::config::Config;

const STRING_LENGTH: usize = 64;

pub struct Client {
    pub(crate) username: String,
    id: u16,
    // The rank of the user, 0x64 for op, 0x00 for normal
    user_type: u8,
    socket: TcpStream,
    current_x: i16,
    current_y: i16,
    current_z: i16
}

impl Client {
    pub async fn new(id: u16, sock: Result<TcpStream, tokio::io::Error>) -> Self {
        Self {
            username: "".to_string(),
            id,
            user_type: 0x00,
            socket: sock.expect("Failed to get socket"),
            current_x: 0,
            current_y: 0,
            current_z: 0
        }
    }

    pub fn get_id(&self) -> u16 {
        self.id
    }

    pub async fn handle_connect(&mut self) -> Result<(), tokio::io::Error> {
        let mut buffer = [0 as u8; 1460];
        self.socket.read(&mut buffer).await?;

        let incoming_packet = Packet::from(buffer.as_ref());
        match incoming_packet {
            ServerBound::PlayerIdentification(protocol, username,
                                              ver_key, _) => {
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
                let data = Packet::into(
                    ClientBound::ServerIdentification(
                        7,
                        name,
                        motd,
                        0x00
                    )
                );
                self.socket.write_all(data.as_slice()).await
                    .expect("Failed to write data");
                self.socket.write_all(Packet::into(ClientBound::LevelInitialize).as_slice())
                    .await
                    .expect("Failed to write data");
                self.socket.write_all(Packet::into(ClientBound::LevelFinalize(50, 50, 50))
                    .as_slice())
                    .await
                    .expect("Failed to write data");
            }
            ServerBound::SetBlock(_, _, _, _, _) => {}
            ServerBound::PositionAndOrientation(
                p_id, x, y, z, yaw, pitch) => {
                // self.tx.send(
                //     Packet::into(ServerBound::PositionAndOrientation(
                //         self.id,
                //         x, y, z, yaw, pitch
                //     ))
                // ).expect("Failed to send PositionAndOrientation");
            }
            ServerBound::Message(_, _) => {}
            ServerBound::UnknownPacket => {
                let msg = String::from_utf8(buffer.to_vec())
                    .expect("Invalid utf8 Message");
                debug!("{}", msg);
            }
        }
        // let received = self.rx.try_recv().unwrap_or(vec![]);
        // debug!("{:x?}", received);
        Ok(())
    }
}