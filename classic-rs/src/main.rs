use tokio::net::{TcpListener};
use tokio::time::{Instant, Duration};
use tokio::sync::Mutex;
use tokio::signal::ctrl_c;
use crossbeam_channel::{Receiver, Sender, unbounded};
use fern::colors::{Color, ColoredLevelConfig};
use log::{info, debug, error, warn, Level};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::sync::{Arc};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};
use mc_worlds::classic::ClassicWorld;

mod client;
mod config;

use client::Client;
use config::{Config, load_whitelist};

struct Server {
    protocol: u8,
    salt: String,
    mo_heartbeat: Arc<Mutex<mineonline_api::heartbeat::Heartbeat>>,
    m_heartbeat: Arc<Mutex<mojang_api::heartbeat::Heartbeat>>,
    running: Arc<AtomicBool>,
    beatdate: Arc<AtomicBool>,
    client_rx: Receiver<Client>,
    network_rx: Receiver<(u8, Vec<ClientBound>)>,
    network_tx: Sender<(u8, Vec<ClientBound>)>,
    world: Arc<Mutex<ClassicWorld>>,
    config: Config,
    clients: Vec<Client>,
    usernames: Vec<String>,
}

impl Server {
    pub async fn new() -> Self {
        let config = Config::get();
        let salt: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
        let running = Arc::new(AtomicBool::new(false));

        let r = running.clone();

        tokio::spawn(async move {
            ctrl_c().await.expect("Failed to listen for event");
            r.store(false, Ordering::SeqCst);
        });
        let world: Arc<Mutex<ClassicWorld>>  = Arc::new(Mutex::new(ClassicWorld::get_or_create(
            &config.map.name, &config.map.creator_username,
            config.map.x_width, config.map.y_height, config.map.z_depth).await));

        // #[cfg(feature = "mineonline_api")]
        let mut mo_heartbeat = Arc::new(Mutex::new(mineonline_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mineonline.url,
            &config.server.ip,
            config.server.port,
            &config.server.name,
            &config.server.motd,
            config.server.public,
            config.server.max_players,
            config.server.online_mode,
            "1FD3397652112BB9E01E49DFE3E47893",
            config.server.whitelisted,
        )));

        // #[cfg(feature = "mojang_api")]
        let mut m_heartbeat = Arc::new(Mutex::new(mojang_api::heartbeat::Heartbeat::new(
            &config.heartbeat.mojang.url,
            &config.server.ip,
            config.server.port,
            &config.server.name,
            config.server.public,
            config.server.max_players,
            config.server.online_mode,
            &salt,
            7
        )));

        if config.heartbeat.enabled {
            // #[cfg(feature = "mineonline_api")]
            if config.heartbeat.mineonline.active {
                let mut mo_beat = mo_heartbeat.lock().await;
                mo_beat.build_request();
                mo_beat.beat().await;
                drop(mo_beat);
            }
            // #[cfg(feature = "mojang_api")]
            if config.heartbeat.mojang.active {
                let mut m_beat = m_heartbeat.lock().await;
                m_beat.build_request();
                m_beat.beat().await;
                drop(m_beat);
            }
        }

        let (tx, rx) = crossbeam_channel::unbounded::<Client>();
        let local_ip = config.server.local_ip.clone();
        let port = config.server.port.clone();
        let tx_clone = tx.clone();
        let (n_tx, n_rx) = crossbeam_channel::unbounded::<(u8, Vec<ClientBound>)>();
        let n_tx_clone = n_tx.clone();

        let r = running.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("{}:{:#?}", local_ip, port)).
                await.expect("Failed to bind");
            r.store(true, Ordering::SeqCst);
            Server::listen(listener, tx_clone, n_tx_clone).await.expect("Failed to listen");
        });

        let beatdate = Arc::new(AtomicBool::new(false));
        if config.heartbeat.enabled {
            let r = running.clone();
            let mo_beat = mo_heartbeat.clone();
            let m_beat = m_heartbeat.clone();
            let bd = beatdate.clone();

            Server::spawn_heartbeats(r, mo_beat, m_beat, bd).await;
        }

        Self {
            protocol: 7,
            salt,
            mo_heartbeat,
            m_heartbeat,
            beatdate,
            running,
            client_rx: rx,
            network_rx: n_rx,
            network_tx: n_tx,
            world,
            config: Config::get(),
            clients: Vec::new(),
            usernames: Vec::new(),
        }
    }

    async fn run(&mut self) -> Result<(), tokio::io::Error> {
        if !self.running.clone().load(Ordering::SeqCst) {
            panic!("Failed to bind to port {:#}", self.config.server.port);
        }
        info!("Server Running at {}:{:#}", self.config.server.ip, self.config.server.port);
        let mut save = Instant::now();
        while self.running.load(Ordering::SeqCst) {
            let timer = Instant::now();

            self.update_network().await;
            self.update_game().await;

            if timer.elapsed().as_millis() > 250 {
                warn!("Last tick took {:?}", timer.elapsed());
            }


            if save.elapsed().as_secs() >= (self.config.server.save_interval * 60) as u64 {
                self.save_world().await;
                save = Instant::now();
            }
        }

        info!("Disconnecting all Clients..");
        let start_disconnect = Instant::now();
        for i in 0..self.clients.len() {
            info!("Disconnecting {}", self.clients[i].username);
            self.clients[i].disconnect(&"Server shutting down")
                .await.expect("Failed to disconnect user");
        }
        info!("Disconnecting took {:?}", start_disconnect.elapsed());

        info!("Saving World...");
        let start_save = Instant::now();
        self.world.lock().await.save_crs_file().await;
        info!("Saving took {:?}", start_save.elapsed());

        if self.config.heartbeat.enabled {
            // #[cfg(feature = "mineonline_api")]
            let mo_beat = &self.mo_heartbeat.lock().await;
            mineonline_api::heartbeat::Heartbeat::delete(&mo_beat.get_url(),
                                                         &mo_beat.get_uuid()).await
                .expect("Failed to send delete request");
        }

        Ok(())
    }

    async fn update_game(&mut self) {

    }

    async fn update_network(&mut self) {
        let mut packet_buffer: Vec<(u8, Vec<ClientBound>)> = vec![(0, Vec::new())];
        let mut player_cleanup: Vec<usize> = vec![];
        let mut fresh_clients: Vec<(u8, usize)> = vec![];
        loop {
            match self.client_rx.try_recv() {
                Ok(client) => {
                    self.clients.push(client);
                    self.beatdate.store(true, Ordering::SeqCst);
                },
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
        loop {
            match self.network_rx.try_recv() {
                Ok(packets) => packet_buffer.push(packets),
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }

        let mut clients = &mut self.clients;

        for c_pos in 0..clients.len() {
            let client = &mut clients[c_pos];
            if client.username != "" && !self.usernames.contains(&client.username) {
                fresh_clients.push((client.get_id(), c_pos));
                self.usernames.push(client.username.clone());
                self.beatdate.store(true, Ordering::SeqCst);
            }
            let mut closed = false;
            match client.handle_connect(&self.salt, self.world.clone()).await {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == tokio::io::ErrorKind::ConnectionReset ||
                        e.kind() == tokio::io::ErrorKind::ConnectionAborted {
                        if !client.username.is_empty() {
                            self.network_tx.send((client.get_id(), client.despawn_self().to_vec())).unwrap();
                        }
                        closed = true;
                    } else {
                        panic!("{}", e);
                    }
                }
            }
            if !closed {
                for i in 0..packet_buffer.len() {
                    let packets = &packet_buffer[i];
                    if packets.0 != client.get_id() {
                        client.write_packets(packets.1.clone()).await;
                    }
                }
            } else {
                for i in 0..self.usernames.len() {
                    if i >= self.usernames.len() {
                        break;
                    }
                    if self.usernames[i] == client.username {
                        self.usernames.remove(i);
                    }
                }
                player_cleanup.push(c_pos);
                self.beatdate.store(true, Ordering::SeqCst);
            }
        }

        if fresh_clients.len() > 0 {
            for f_client in &fresh_clients {
                let mut packets: Vec<ClientBound> = vec![];
                for c in &mut self.clients {
                    if c.get_id() != f_client.0 {
                        packets.push(c.spawn_self().await);
                    }
                }
                let c = &mut self.clients[f_client.1];
                c.write_packets(packets).await;
            }
        }

        for id in player_cleanup {
            self.clients.remove(id);
        }
        if self.beatdate.clone().load(Ordering::SeqCst) {
            if self.config.heartbeat.enabled {
                if self.config.heartbeat.mineonline.active {
                    let mut mo_beat = self.mo_heartbeat.lock().await;
                    mo_beat.update_player_names(&self.usernames);
                    mo_beat.update_users(self.clients.len() as u16);
                    drop(mo_beat);
                }
                if self.config.heartbeat.mojang.active {
                    let mut m_beat = self.m_heartbeat.lock().await;
                    m_beat.update_users(self.clients.len() as u16);
                    drop(m_beat);
                }
            }
            self.beatdate.store(false, Ordering::SeqCst);
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

    async fn spawn_heartbeats(running: Arc<AtomicBool>, mo_heartbeat: Arc<Mutex<mineonline_api::heartbeat::Heartbeat>>,
                              m_heartbeat: Arc<Mutex<mojang_api::heartbeat::Heartbeat>>, beatdate: Arc<AtomicBool>) {
        tokio::spawn(async move {
            let mut duration = Instant::now();
            let config = Config::get();
            while running.load(Ordering::SeqCst) {
                if duration.elapsed().as_secs() % 40 == 0 || beatdate.load(Ordering::SeqCst) {
                    let mut mo_heartbeat = mo_heartbeat.lock().await;
                    let mut m_heartbeat = m_heartbeat.lock().await;
                    if config.heartbeat.mineonline.active {
                        mo_heartbeat.build_request();
                        mo_heartbeat.beat().await;
                    }
                    if config.heartbeat.mojang.active {
                        m_heartbeat.build_request();
                        m_heartbeat.beat().await;
                    }
                    // beatdate.store(false, Ordering::SeqCst);
                    drop(mo_heartbeat);
                    drop(m_heartbeat);
                }
                duration = Instant::now();
            }
        });
    }

    async fn save_world(&mut self) {
        let w_lock = self.world.clone();
        let world = w_lock.lock().await;
        for c in &mut self.clients {
            c.send_message(
                c.build_message("Console", 255, "Saving World..").await).await;
        }
        world.save_crs_file().await;
        for c in &mut self.clients {
            c.send_message(
                c.build_message("Console", 255, "Saving Complete").await).await;
        }
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
    }));

    Ok(())
}
