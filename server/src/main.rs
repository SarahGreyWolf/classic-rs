use tokio::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use futures::stream::StreamExt;

use mineonline_api::heartbeat::Heartbeat;
use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};

struct Server {
    ip: String,
    port: u16,
    listener: TcpListener,
    heartbeat: Heartbeat
}

impl Server {
    pub async fn new(ip: &str, port: u16) -> Self {
        println!("Server Running at {}:{:#}", ip, port);
        let listener = TcpListener::bind(format!("{}:{:#}", ip, port)).await.expect("Failed to bind");
        let heartbeat = Heartbeat::new(
            "192.168.0.14",
            25565,
            "Sarah's Pipe Dream",
            false,
            8,
            true,
            "90632803F45C15164587256A08C0ECB4",
            true
        );
        Self {
            ip: ip.to_string(),
            port,
            listener,
            heartbeat
        }
    }

    pub async fn listen(&mut self) -> Result<(), tokio::io::Error> {
        let mut incoming = self.listener.incoming();
        while let Some(conn) = incoming.next().await {
            match conn {
                Err(e) => eprintln!("Accept Failed = {:?}", e),
                Ok(mut sock) => {
                    let (mut reader, mut writer) = sock.split();
                    let mut buffer = [0 as u8; 1027];
                    reader.read(&mut buffer).await?;
                    let incoming_packet = Packet::from(buffer.as_ref());
                    match incoming_packet {
                        ServerBound::PlayerIdentification(protocol, msg) => {}
                        ServerBound::SetBlock(_, _, _, _, _) => {}
                        ServerBound::PositionAndOrientation(_, _, _, _, _, _) => {}
                        ServerBound::Message(_, _) => {}
                        ServerBound::UnknownPacket => {
                            let msg = String::from_utf8(buffer.to_vec())
                                .expect("Invalid utf8 Message").replace("\u{20}", "");
                            println!("{}", msg);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    

    // hearbeat.update_whitelist(vec!["SarahGreyWolf".to_string()], vec![]);
    // hearbeat.build_mineonline_request();
    // hearbeat.beat().await;
    let mut server = Server::new("127.0.0.1", 25565).await;
    server.listen().await?;

    Ok(())
}