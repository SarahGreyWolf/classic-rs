use tokio::net::{TcpListener};
use tokio::time::{Instant, Duration};
use tokio::signal::ctrl_c;
use flume::{Receiver, Sender};
use fern::colors::{Color, ColoredLevelConfig};
use log::{info, debug, error, warn, Level};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use specs::{World, WorldExt, DispatcherBuilder, Builder};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

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
use std::sync::atomic::{AtomicBool, Ordering};

struct Server {
    protocol: u8,
    salt: String,
    mo_heartbeat: mineonline_api::heartbeat::Heartbeat,
    m_heartbeat: mojang_api::heartbeat::Heartbeat,
    running: Arc<AtomicBool>,
    beatdate: bool,
    client_rx: Receiver<Client>,
    network_rx: Receiver<(u8, Vec<ClientBound>)>,
    network_tx: Sender<(u8, Vec<ClientBound>)>,
    world: Arc<Mutex<ClassicWorld>>,
    // ecs_world: World,
    config: Config,
    clients: Vec<Client>,
    usernames: Vec<String>,
}

impl Server {
    pub async fn new() -> Self {
        let config = Config::get();
        let salt: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
        let world: Arc<Mutex<ClassicWorld>>  = Arc::new(Mutex::new(ClassicWorld::new(
            &config.map.name, config.map.width, config.map.height, config.map.depth)));
        #[cfg(feature = "mineonline_api")]
        let mut mo_heartbeat = mineonline_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mineonline.url,
            &config.server.ip,
            config.server.port,
            &config.server.name,
            config.server.public,
            config.server.max_players,
            config.server.online_mode,
            "90632803F45C15164587256A08C0ECB4",
            config.server.whitelisted,
        );

        #[cfg(feature = "mojang_api")]
        let mut m_heartbeat = mojang_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mojang.url,
            &config.server.ip,
            config.server.port,
            &config.server.name,
            config.server.public,
            config.server.max_players,
            config.server.online_mode,
            &salt,
            7
        );

        #[cfg(feature = "mineonline_api")]
        if config.heartbeat.mineonline.active {
            mo_heartbeat.build_request();
            mo_heartbeat.beat().await;
        }
        #[cfg(feature = "mojang_api")]
        if config.heartbeat.mojang.active {
            m_heartbeat.build_request();
            m_heartbeat.beat().await;
        }

        let (tx, rx) = flume::unbounded::<Client>();
        let local_ip = config.server.local_ip.clone();
        let port = config.server.port.clone();
        let tx_clone = tx.clone();
        let (n_tx, n_rx) = flume::unbounded::<(u8, Vec<ClientBound>)>();
        let n_tx_clone = n_tx.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("{}:{:#}", local_ip, port, )).
                await.expect("Failed to bind");
            Server::listen(listener, tx_clone, n_tx_clone).await.expect("Failed to listen");
        });
        // let mut ecs_world = ecs::initialise_world();
        // let mut dispatcher = DispatcherBuilder::new()
        //     .with(ecs::systems::NetworkReadSys, "net_sys", &[]).build();
        // dispatcher.setup(&mut world);
        // dispatcher.dispatch(&mut world);
        // world.maintain();

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        #[cfg(feature = "mineonline_api")]
        let uuid = mo_heartbeat.get_uuid().to_string().clone();
        let url = mo_heartbeat.get_url().to_string().clone();

        tokio::spawn(async move {
            ctrl_c().await.expect("Failed to listen for event");

            r.store(false, Ordering::SeqCst);
        });

        Self {
            protocol: 7,
            salt,
            mo_heartbeat,
            m_heartbeat,
            beatdate: false,
            running,
            client_rx: rx,
            network_rx: n_rx,
            network_tx: n_tx,
            world,
            // ecs_world,
            config: Config::get(),
            clients: Vec::new(),
            usernames: Vec::new(),
        }
    }

    async fn run(&mut self) -> Result<(), tokio::io::Error> {

        info!("Server Running at {}:{:#}", self.config.server.ip, self.config.server.port);
        let start = Instant::now();
        let mut end = Instant::now();
        while self.running.load(Ordering::SeqCst) {
            let duration = end.duration_since(start);

            self.update_network().await;
            self.update_game().await;
            if (duration.as_secs() % 40) == 0 || self.beatdate {
                if self.config.heartbeat.mineonline.active {
                    self.mo_heartbeat.update_players(&self.usernames);
                    self.mo_heartbeat.update_users(self.clients.len() as u16);
                    self.mo_heartbeat.build_request();
                    self.mo_heartbeat.beat().await;
                }
                if self.config.heartbeat.mojang.active {
                    self.m_heartbeat.update_users(self.clients.len() as u16);
                    self.m_heartbeat.build_request();
                    self.m_heartbeat.beat().await;
                }
                self.beatdate = false;
            }

            end = Instant::now();
        }

        info!("Disconnecting all Clients");
        for i in 0..self.clients.len() {
            info!("Disconnecting {}", self.clients[i].username);
            self.clients[i].disconnect(&"Server shutting down")
                .await.expect("Failed to disconnect user");
        }
        #[cfg(feature = "mineonline_api")]
        if self.config.heartbeat.mineonline.active {
            self.mo_heartbeat.update_players(&Vec::new());
            self.mo_heartbeat.update_users(0);
            self.mo_heartbeat.build_request();
            self.mo_heartbeat.beat().await;
        }
        #[cfg(feature = "mojang_api")]
        if self.config.heartbeat.mojang.active {
            self.m_heartbeat.update_users(0);
            self.m_heartbeat.build_request();
            self.m_heartbeat.beat().await;
        }

        #[cfg(feature = "mineonline_api")]
        mineonline_api::heartbeat::Heartbeat::delete(&self.mo_heartbeat.get_url(),
                                                     &self.mo_heartbeat.get_uuid()).await
            .expect("Failed to send delete request");

        Ok(())
    }

    async fn update_game(&mut self) {

    }

    async fn update_network(&mut self) {
        let mut packet_buffer: Vec<(u8, Vec<ClientBound>)> = vec![(0, Vec::new())];
        loop {
            match self.client_rx.try_recv() {
                Ok(client) => {
                    self.clients.push(client);
                    self.beatdate = true;
                },
                Err(flume::TryRecvError::Empty) => break,
                Err(flume::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
        loop {
            match self.network_rx.try_recv() {
                Ok(packets) => packet_buffer.push(packets),
                Err(flume::TryRecvError::Empty) => break,
                Err(flume::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }

        let mut clients = &mut self.clients;

        for mut c_pos in 0..clients.len() {
            let client = &mut clients[c_pos];
            if client.username != "" && !self.usernames.contains(&client.username) {
                self.usernames.push(client.username.clone());
                self.beatdate = true;
            }
            let mut closed = false;
            match client.handle_connect(self.world.clone()).await {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == tokio::io::ErrorKind::ConnectionReset {
                        info!("{} has left the server", client.username);
                        closed = true;
                    } else if e.kind() == tokio::io::ErrorKind::ConnectionAborted {
                        info!("{} has left the server", client.username);
                        closed = true;
                    } else {
                        panic!("{}", e);
                    }
                }
            }
            for i in 0..packet_buffer.len() {
                let packets = &packet_buffer[i];
                if packets.0 != client.get_id() {
                    client.write_packets(&packets.1).await;
                }
            }
            if closed {
                for i in 0..self.usernames.len() {
                    if self.usernames[i] == client.username {
                        self.usernames.remove(i);
                    }
                }
                clients.remove(c_pos);
                self.beatdate = true;
                c_pos = c_pos - 1;
                if c_pos + 1 == clients.len() {
                    break;
                }
            }
        }
    }

    async fn listen(mut listener: TcpListener, tx: Sender<Client>, n_tx: Sender<(u8, Vec<ClientBound>)>)
        -> Result<(), tokio::io::Error> {
        let mut id: u8 = 0;
        while let Ok((stream, addr)) = listener.accept().await {
            let client = Client::new(id, stream, n_tx.clone()).await;
            if tx.send(client).is_err() {
                panic!("Failed to send client");
            }
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
    let datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M");
    let log_file_path: PathBuf = PathBuf::from(&format!("./logs/{}.log", datetime));
    if tokio::fs::read_dir("./logs").await.is_err() {
        tokio::fs::create_dir("./logs").await.expect("Failed to create logs folder");
        tokio::fs::File::open(&log_file_path).await.unwrap_or(
            tokio::fs::File::create(&log_file_path).await.expect("Failed to create log file")
        );
    }
    fern::Dispatch::new()
        .chain(std::io::stdout())
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}]{}[{}] {}",
                // This will color the log level only, not the whole line. Just a touch.
                colors.color(record.level()),
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.module_path().unwrap(),
                message
            ))
        })
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("want", log::LevelFilter::Info)
        .level_for("mio", log::LevelFilter::Info)
        .level_for("reqwest", log::LevelFilter::Info)
        .level_for("tokio", log::LevelFilter::Info)
        .chain(fern::log_file(log_file_path)?)
        .apply()
        .unwrap();
    std::panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info.to_string());
        // let backtrace = Backtrace::new();
        // error!("{}\n{:?}", panic_info.to_string(), backtrace);
    }));

    Ok(())
}
