use std::fs::read_to_string;
use std::fs::write;
use std::str::FromStr;
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};
use toml::{to_string, from_str};


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
            max_players: 8
        };
        let out = to_string(&config)
            .expect("Failed to convert to TOML string");
        write(path, &out).expect("Failed to write to server.toml");
        out
    }

    pub fn get() -> Self {
        let path = PathBuf::from_str("./server.toml").expect("Could not get path");
        let file = &read_to_string(path).unwrap_or(Config::create());

        from_str(file).expect("Failed to parse TOML to Config")
    }
}