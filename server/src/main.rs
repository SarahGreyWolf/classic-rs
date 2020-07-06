use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use std::time::SystemTime;
use flume::{Receiver, Sender};
use fern::colors::{Color, ColoredLevelConfig};
use log::{info, debug, error, warn};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

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
    salt: String,
    mo_heartbeat: mineonline_api::heartbeat::Heartbeat,
    m_heartbeat: mojang_api::heartbeat::Heartbeat,
    listener: TcpListener,
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    clients: Box::<Vec<Sender<Vec<u8>>>>,
    config: Config,
}

impl Server {
    pub fn new() -> Self {
        let config = Config::get();
        let salt: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
        let listener = TcpListener::bind(format!("{}:{:#}", config.ip, config.port))
            .expect("Failed to bind");

        let mut mo_heartbeat = mineonline_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mineonline.url,
            &config.ip,
            config.port,
            &config.name,
            config.public,
            config.max_players,
            config.online_mode,
            "90632803F45C15164587256A08C0ECB4",
            config.whitelisted
        );
        mo_heartbeat.build_request();

        let mut m_heartbeat = mojang_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mojang.url,
            &config.ip,
            config.port,
            &config.name,
            config.public,
            config.max_players,
            config.online_mode,
            &salt,
            7,
            config.whitelisted
        );
        m_heartbeat.build_request();
        if config.heartbeat.mineonline.active {
            mo_heartbeat.beat();
        }
        if config.heartbeat.mojang.active {
            m_heartbeat.beat();
        }

        let (tx, rx) = flume::unbounded::<Vec<u8>>();
        info!("Server Running at {}:{:#}", config.ip, config.port);
        // heartbeat.beat();
        Self {
            ip: config.ip,
            port: config.port,
            name: config.name,
            motd: config.motd,
            protocol: 7,
            salt,
            mo_heartbeat,
            m_heartbeat,
            listener,
            tx,
            rx,
            clients: Box::new(Vec::new()),
            config: Config::get(),
        }
    }

    fn game_loop(mut self) {
        spawn(move || loop {
            let received = self.rx.try_recv().expect("Failed to receive");
            debug!("FUCK");
            let packet: ServerBound = Packet::from(received.as_slice());
            let clients = &self.clients;
            match packet {
                ServerBound::PlayerIdentification(protocol, username,
                                                  ver_key, _) => {},
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
            if self.config.heartbeat.mineonline.active {
                self.mo_heartbeat.beat();
            }
            if self.config.heartbeat.mojang.active {
                self.m_heartbeat.beat();
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