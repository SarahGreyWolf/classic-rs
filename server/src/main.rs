use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use tokio::stream::StreamExt;
use tokio::time::{Instant, Duration};
use flume::{Receiver, Sender};
use fern::colors::{Color, ColoredLevelConfig};
use log::{info, debug, error, warn, Level};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use specs::{World, WorldExt, DispatcherBuilder, Builder};
use backtrace::Backtrace;
use std::sync::{Arc, Mutex};

use std::fs::File;
use std::io::{BufReader, Read};

use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};
use mc_worlds::classic::ClassicWorld;
use grey_mc_api::event;

mod client;
mod config;
mod ecs;

use client::Client;
use config::Config;
use ecs::components::{common, player, entity};

struct Server {
    protocol: u8,
    salt: String,
    mo_heartbeat: mineonline_api::heartbeat::Heartbeat,
    m_heartbeat: mojang_api::heartbeat::Heartbeat,
    client_rx: Receiver<Client>,
    world: Arc<Mutex<ClassicWorld>>,
    ecs_world: World,
    config: Config,
    clients: Vec<Client>,
}

impl Server {
    pub async fn new() -> Self {
        let config = Config::get();
        let salt: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();

        let mut mo_heartbeat = mineonline_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mineonline.url,
            &config.server.ip,
            config.server.port,
            &config.server.name,
            config.server.public,
            config.server.max_players,
            config.server.online_mode,
            "90632803F45C15164587256A08C0ECB4",
            config.server.whitelisted
        );
        mo_heartbeat.build_request();

        let mut m_heartbeat = mojang_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mojang.url,
            &config.server.ip,
            config.server.port,
            &config.server.name,
            config.server.public,
            config.server.max_players,
            config.server.online_mode,
            &salt,
            7,
            config.server.whitelisted
        );
        m_heartbeat.build_request();
        if config.heartbeat.mineonline.active {
            mo_heartbeat.beat().await;
        }
        if config.heartbeat.mojang.active {
            m_heartbeat.beat().await;
        }

        let (tx, rx) = flume::unbounded::<Client>();
        let local_ip = config.server.local_ip.clone();
        let port = config.server.port.clone();
        let tx_clone = tx.clone();
        core::mem::forget(tx);
        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("{}:{:#}", local_ip, port)).
                await.expect("Failed to bind");
            Server::listen(listener, tx_clone).await.expect("Failed to listen");
        });
        let mut ecs_world = ecs::initialise_world();
        // let mut dispatcher = DispatcherBuilder::new()
        //     .with(ecs::systems::NetworkReadSys, "net_sys", &[]).build();
        // dispatcher.setup(&mut world);
        // dispatcher.dispatch(&mut world);
        // world.maintain();

        Self {
            protocol: 7,
            salt,
            mo_heartbeat,
            m_heartbeat,
            client_rx: rx,
            world: Arc::new(Mutex::new(ClassicWorld::new(&"SarahWorld",10, 5, 10))),
            ecs_world,
            config: Config::get(),
            clients: Vec::new(),
        }
    }

    async fn run(&mut self) -> Result<(), tokio::io::Error> {

        info!("Server Running at {}:{:#}", self.config.server.ip, self.config.server.port);
        let start = Instant::now();
        let mut end = Instant::now();
        loop {
            let duration = end.duration_since(start);

            self.update_network().await;
            self.update_game().await;
            if (duration.as_secs() % 40) == 0 {
                if self.config.heartbeat.mineonline.active {
                    self.mo_heartbeat.build_request();
                    self.mo_heartbeat.beat().await;
                }
                if self.config.heartbeat.mojang.active {
                    self.m_heartbeat.build_request();
                    self.m_heartbeat.beat().await;
                }
            }
            end = Instant::now();
        }

        Ok(())
    }

    async fn update_game(&mut self) {

    }

    async fn update_network(&mut self) {
        loop {
            match self.client_rx.try_recv() {
                Ok(client) => self.clients.push(client),
                Err(flume::TryRecvError::Empty) => break,
                Err(flume::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
        for c_pos in 0..self.clients.len() {
            match self.clients[c_pos].handle_connect(self.world.clone()).await {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == tokio::io::ErrorKind::ConnectionReset {
                        info!("Player {} has disconnected", self.clients[c_pos].username);
                        self.clients.remove(c_pos);
                    } else if e.kind() == tokio::io::ErrorKind::ConnectionAborted {
                        info!("Player {} has disconnected", self.clients[c_pos].username);
                        self.clients.remove(c_pos);
                    }else {
                        panic!("{}", e);
                    }
                }
            }
        }
    }

    async fn listen(mut listener: TcpListener, tx: Sender<Client>)
        -> Result<(), tokio::io::Error> {
        let mut id: u16 = 0;
        while let Ok((stream, addr)) = listener.accept().await {
            let client = Client::new(id, stream).await;
            if tx.send(client).is_err() {
                panic!("Failed to send client");
            }
            info!("{:#}", id);
            id += 1;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    init_logging().await?;
    let mut server = Server::new().await;
    server.run().await.expect("Server loop Failed");



    Ok(())
}

async fn init_logging() -> Result<(), tokio::io::Error> {
    let colors = ColoredLevelConfig::new()
        .info(Color::Magenta)
        .error(Color::BrightRed);
    if tokio::fs::read_dir("./logs").await.is_err() {
        tokio::fs::create_dir("./logs").await.expect("Failed to create logs folder");
    }
    fern::Dispatch::new()
        .chain(std::io::stdout())
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}]{} {}",
                // This will color the log level only, not the whole line. Just a touch.
                colors.color(record.level()),
                record.module_path().unwrap(),
                chrono::Utc::now().format("[%Y-%m-%d %H:%M:%S]"),
                message
            ))
        })
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("want", log::LevelFilter::Info)
        .level_for("mio", log::LevelFilter::Info)
        .level_for("reqwest", log::LevelFilter::Info)
        .level_for("tokio", log::LevelFilter::Info)
        .chain(fern::log_file("./logs/latest.log")?)
        .apply()
        .unwrap();
    std::panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info.to_string());
        // let backtrace = Backtrace::new();
        // error!("{}\n{:?}", panic_info.to_string(), backtrace);
    }));

    Ok(())
}