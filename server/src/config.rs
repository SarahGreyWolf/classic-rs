use std::fs::read_to_string;
use std::fs::write;
use std::str::FromStr;
use std::path::PathBuf;
use std::io::Error;
use serde_derive::{Deserialize, Serialize};
use toml::{to_string, from_str};
use log::{debug};

#[derive(Serialize, Deserialize)]
pub struct MineOnline {
    pub active: bool,
    pub url: String,
}

impl Default for MineOnline {
    fn default() -> Self {
        Self {
            active: true,
            url: "http://mineonline.codie.gg/mineonline/listserver.jsp".to_string()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mojang {
    pub active: bool,
    pub url: String,
}

impl Default for Mojang {
    fn default() -> Self {
        Self {
            active: false,
            url: "http://www.minecraft.net/heartbeat.jsp".to_string()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Heartbeat {
    pub mineonline: MineOnline,
    pub mojang: Mojang
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            mineonline: MineOnline::default(),
            mojang: Mojang::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    pub name: String,
    pub motd: String,
    pub public: bool,
    pub online_mode: bool,
    pub whitelisted: bool,
    pub max_players: u16,
    pub heartbeat: Heartbeat
}

impl Config {
    pub fn create() -> String {
        let path = PathBuf::from_str("./server.toml").expect("Could not get path");
        let config = Self {
            ip: "0.0.0.0".to_string(),
            port: 25565,
            name: "A Minecraft Server".to_string(),
            motd: "A Minecraft Server".to_string(),
            public: true,
            online_mode: true,
            whitelisted: false,
            max_players: 8,
            heartbeat: Heartbeat::default(),
        };
        let out = to_string(&config)
            .expect("Failed to convert to TOML string");
        write(path, &out).expect("Failed to write to server.toml");
        out
    }

    pub fn get() -> Self {
        let path = PathBuf::from_str("./server.toml").expect("Could not get path");
        let mut file = "".to_string();
        match read_to_string(path) {
            Ok(f) => {
                file = f;
            },
            Err(e) => {
                debug!("Error occurred reading string: {}", e);
                file = Config::create();
            },
        }

        from_str(&file).expect("Failed to parse TOML to Config")
    }
}