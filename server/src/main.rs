use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use std::time::SystemTime;
use flume::{Receiver, Sender};

use fern::colors::{Color, ColoredLevelConfig};
use log::{info, debug, error, warn};

use mineonline_api::heartbeat::Heartbeat;
use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};
use grey_mc_api::event;

mod client;
mod config;

use client::Client;
use config::Config;

struct Server {
    ip: String,
    port: u16,
    name: String,
    motd: String,
    protocol: u8,
    heartbeat: Heartbeat,
    listener: TcpListener,
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    clients: Box::<Vec<Sender<Vec<u8>>>>
}

impl Server {
    pub fn new() -> Self {
        let config = Config::get();

        let listener = TcpListener::bind(format!("{}:{:#}", config.ip, config.port))
            .expect("Failed to bind");
        let mut heartbeat = Heartbeat::new(
            &config.ip,
            config.port,
            &config.name,
            config.public,
            config.max_players,
            config.online_mode,
            "90632803F45C15164587256A08C0ECB4",
            config.whitelisted
        );
        heartbeat.build_mineonline_request();
        let (tx, rx) = flume::unbounded::<Vec<u8>>();
        info!("Server Running at {}:{:#}", config.ip, config.port);
        // heartbeat.beat();
        Self {
            ip: config.ip,
            port: config.port,
            name: config.name,
            motd: config.motd,
            protocol: 7,
            heartbeat,
            listener,
            tx,
            rx,
            clients: Box::new(Vec::new())
        }
    }

    fn game_loop(self) {
        spawn(move || loop {
            let received = self.rx.try_recv().expect("Failed to receive");
            debug!("FUCK");
            let packet: ServerBound = Packet::from(received.as_slice());
            let clients = &self.clients;
            match packet {
                ServerBound::PlayerIdentification(protocol, username) => {},
                ServerBound::SetBlock(_, _, _, _, _) => {}
                ServerBound::PositionAndOrientation(
                    p_id, x, y, z, yaw, pitch) => {
                    for i in 0..clients.len() {
                        info!("{:x}", p_id);
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
    let colors = ColoredLevelConfig::new()
        .info(Color::Magenta)
        .error(Color::BrightRed);
    fern::Dispatch::new()
        .chain(std::io::stdout())
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}]{} {}",
                // This will color the log level only, not the whole line. Just a touch.
                colors.color(record.level()),
                chrono::Utc::now().format("[%Y-%m-%d %H:%M:%S]"),
                message
            ))
        })
        .apply()
        .unwrap();
    let server = Server::new();
    server.listen().expect("Failed to listen");
    Ok(())
}