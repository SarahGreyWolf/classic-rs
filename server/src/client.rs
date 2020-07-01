use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

use flume::{Receiver, Sender};

use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};
use std::io::Write;

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

    pub fn get_id(self) -> u8 {
        self.id
    }

    pub fn handle_connect(mut self) -> Result<(), std::io::Error> {
        match self.socket {
            Err(e) => eprintln!("Accept Failed = {:?}", e),
            Ok(mut sock) => {
                let mut buffer = [0 as u8; 1460];
                sock.peek(&mut buffer);

                let incoming_packet = Packet::from(buffer.as_ref());
                match incoming_packet {
                    ServerBound::PlayerIdentification(protocol, username) => {
                        self.username = username;
                        println!("{}", self.username);
                        let data = Packet::into(
                            // TODO: Get this from config or server struct
                            ClientBound::ServerIdentification(
                                7,
                                "Sarah's Pipe Dream".to_string(),
                                "We live here damnit!".to_string(),
                                0x64
                            )
                        );
                        println!("{:x?}", data);
                        sock.write(data.as_slice()).expect("Failed to write data");
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
                        println!("{}", msg);
                    }
                }
                let received = self.rx.try_recv().expect("Failed to receive data");
                println!("{:x?}", received);
            }
        }
        Ok(())
    }
}