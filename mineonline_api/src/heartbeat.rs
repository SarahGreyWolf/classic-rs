//! # Heartbeat
//! Used for sending Minecraft Heartbeats to [Mineonline](http://mineonline.codie.gg)

use json::{parse, stringify, JsonValue};
use json::number::Number;
use json::object::Object;
use reqwest::{Body, Url, StatusCode};
use log::{debug};
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
struct Response {
    uuid: String
}

/// Heartbeat Object
pub struct Heartbeat {
    uuid: String,
    url: String,
    ip: String,
    port: u16,
    name: String,
    public: String,
    max_users: u16,
    online: String,
    client_hash: String,
    users: u16,
    players_list: Vec<String>,
    whitelisted: bool,
    request: String,
}

impl Heartbeat {
    /// Create a Heartbeat Object.
    ///
    pub fn new(url: &str, ip: &str, port: u16, name: &str, public: bool, max_players: u16, online: bool,
               client_hash: &str, whitelisted: bool) -> Self {
        Self {
            uuid: "".to_string(),
            url: url.to_string(),
            ip: ip.to_string(),
            port,
            name: name.to_string(),
            public: public.to_string(),
            max_users: max_players,
            online: online.to_string(),
            client_hash: client_hash.to_string(),
            users: 0,
            players_list: vec![],
            whitelisted,
            request: "".to_string(),
        }
    }
    /// Update the number of users currently connected to the server in the heartbeat.
    pub fn update_users(&mut self, user_count: u16) {
        self.users = user_count;
    }

    /// Update the usernames of users currently connected to the server in the heartbeat.
    pub fn update_player_names(&mut self, user_names: &Vec<String>) {
        self.players_list = user_names.to_vec();
    }
    /// Builds the request data from the heartbeat.
    pub fn build_request(&mut self) -> String {
        let mut mineonline_json: Object = Object::new();
        mineonline_json.insert("ip",
                               JsonValue::String(self.ip.to_string()));
        mineonline_json.insert("port",
                               JsonValue::String(self.port.to_string()));
        mineonline_json.insert("users",
                               JsonValue::Number(Number::from(self.users)));
        let players: Vec<JsonValue> = self.players_list.iter().map(
            |p| JsonValue::String(String::from(p))).collect();
        mineonline_json.insert("players",
                               JsonValue::Array(players));
        mineonline_json.insert("max",
                               JsonValue::Number(Number::from(self.max_users)));
        mineonline_json.insert("name",
                               JsonValue::String(self.name.to_string()));
        mineonline_json.insert("onlinemode",
                               JsonValue::Boolean(self.online.to_string().parse().unwrap()));
        mineonline_json.insert("md5",
                               JsonValue::String(self.client_hash.to_string()));
        mineonline_json.insert("whitelisted",
                               JsonValue::Boolean(self.whitelisted));
        // TODO: Support Owner Name

        let stringified = stringify(mineonline_json);
        self.request = (&stringified).parse().unwrap();
        stringified

    }

    pub fn get_user_count(&self) -> u16 {
        self.users
    }

    pub fn get_request(&self) -> &str {
        &self.request
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
    pub fn get_url(&self) -> &str {
        &self.url
    }
    /// Causes a heartbeat request to be made to the server
    pub async fn beat(&mut self) {
        let mut retry: bool = true;
        let mut tries: u8 = 0;
        while retry && tries < 5 {
            let request_client = reqwest::Client::new();
            let request = request_client.post(Url::parse(&self.url)
                .expect("Failed to parse to URL").join("/api/servers").unwrap()
            ).header("content-type", "application/json").body(self.request.clone());
            let response = request.send().await.expect("Failed to make post request");
            if response.status() != StatusCode::OK {
                if tries == 5 {
                    panic!("Heartbeat Request Failed: {}", response.status());
                }
                tries += 1;
                std::thread::sleep(std::time::Duration::from_secs(2));
            } else {
                let json_response = response.json::<Response>().await.expect("Failed to parse json");
                self.uuid = json_response.uuid;
                retry = false;
            }
        }
    }
    /// Delete the server from the server list
    pub async fn delete(url: &str, uuid: &str) -> Result<(), std::io::Error> {
        let request_client = reqwest::Client::new();
        let end_url = format!("/api/servers/{}", uuid);
        let request = request_client.delete(Url::parse(url)
            .expect("Failed to parse URL").join(&end_url).unwrap());
        let response = request.send().await.expect("Failed to make post request");
        if response.status() != StatusCode::OK {
            panic!("Heartbeat Request Failed: {}", response.status());
        }
        Ok(())
    }
}