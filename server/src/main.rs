use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

use flume::{Receiver, Sender};

use mineonline_api::heartbeat::Heartbeat;
use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};

mod client;
use client::Client;

struct Server {
    ip: String,
    port: u16,
    listener: TcpListener,
    heartbeat: Heartbeat,
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    clients: Vec<Sender<Vec<u8>>>
}

impl Server {
    pub fn new(ip: &str, port: u16) -> Self {
        println!("Server Running at {}:{:#}", ip, port);
        let listener = TcpListener::bind(format!("{}:{:#}", ip, port))
            .expect("Failed to bind");
        let heartbeat = Heartbeat::new(
            ip,
            port,
            "Sarah's Pipe Dream",
            false,
            8,
            true,
            "90632803F45C15164587256A08C0ECB4",
            true
        );
        let (tx, rx) = flume::unbounded::<Vec<u8>>();
        Self {
            ip: ip.to_string(),
            port,
            listener,
            heartbeat,
            tx,
            rx,
            clients: Vec::new()
        }
    }

    fn game_loop(self) {
        spawn(move || loop {
            let received = self.rx.try_recv().expect("Failed to receive");
            let packet: ServerBound = Packet::from(received.as_slice());
            let clients = &self.clients;
            match packet {
                ServerBound::PlayerIdentification(protocol, username) => {},
                ServerBound::SetBlock(_, _, _, _, _) => {}
                ServerBound::PositionAndOrientation(
                    p_id, x, y, z, yaw, pitch) => {
                    for i in 0..clients.len() {
                        if i == p_id as usize {
                            continue;
                        }else{
                            // clients[0].try_send(
                            //     Packet::into(
                            //         ClientBound::PositionAndOrientationUpdate(
                            //             p_id,
                            //
                            //         )
                            //     )
                            // );
                        }
                    }
                }
                ServerBound::Message(_, _) => {}
                ServerBound::UnknownPacket => {}
            }
        });
    }

    fn listen(mut self) -> Result<(), std::io::Error> {
        let mut incoming = self.listener.incoming();
        while let Some(conn) = incoming.next() {
            let mut tx = self.tx.clone();
            let (mut ctx, mut crx) =
                flume::unbounded::<Vec<u8>>();
            self.clients.push(ctx.clone());
            let id = self.clients.len() as u8;
            let client = Client::new(id.clone(), conn, tx, crx);
            drop(ctx);
            spawn(move || {
                client.handle_connect().expect("Failed to handle exception");
            });
        }
        self.game_loop();
        Ok(())
    }
}
fn main() -> Result<(), std::io::Error> {

    // hearbeat.update_whitelist(vec!["SarahGreyWolf".to_string()], vec![]);
    // hearbeat.build_mineonline_request();
    // hearbeat.beat().await;
    let server = Server::new("127.0.0.1", 25565);
    server.listen().expect("Failed to listen");

    Ok(())
}