//! # Heartbeat
//! Used for sending Minecraft Heartbeats to [Mineonline](http://mineonline.codie.gg)

use json::{parse, stringify, JsonValue};
use json::number::Number;
use json::object::Object;
use reqwest::{Body, Url, StatusCode};
use log::{debug};

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
    whitelisted_users: Vec<String>,
    whitelisted_ips: Vec<String>,
    whitelisted_uuids: Vec<String>,
    banned_users: Vec<String>,
    banned_ips: Vec<String>,
    banned_uuids: Vec<String>,
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
            whitelisted_users: vec![],
            whitelisted_ips: vec![],
            whitelisted_uuids: vec![],
            banned_users: vec![],
            banned_ips: vec![],
            banned_uuids: vec![],
            request: "".to_string(),
        }
    }
    /// Update the number of users currently connected to the server in the heartbeat.
    pub fn update_users(&mut self, user_count: u16) {
        self.users = user_count;
    }

    /// Update the usernames of users currently connected to the server in the heartbeat.
    pub fn update_players(&mut self, user_names: &Vec<String>) {
        self.players_list = user_names.to_vec();
    }

    /// Update the servers ban list in the heartbeat.
    pub fn update_bans(&mut self, banned_users: Vec<String>, banned_ips: Vec<String>, banned_uuids: Vec<String>) {
        self.banned_users = banned_users;
        self.banned_ips = banned_ips;
        self.banned_uuids = banned_uuids;
    }
    /// Update the servers whitelist in the heartbeat.
    pub fn update_whitelist(&mut self, wl_users: Vec<String>, wl_ips: Vec<String>, wl_uuids: Vec<String>) {
        self.whitelisted_users = wl_users;
        self.whitelisted_ips = wl_ips;
        self.whitelisted_uuids = wl_uuids;
    }
    /// Builds the request data from the heartbeat.
    pub fn build_request(&mut self) -> String {
        let mut mineonline_json: Object = Object::new();
        mineonline_json.insert("ip",
                               JsonValue::String(String::from(&self.ip)));
        mineonline_json.insert("port",
                               JsonValue::String(String::from(&self.port.to_string())));
        mineonline_json.insert("users",
                               JsonValue::Number(Number::from(self.users)));
        let players: Vec<JsonValue> = self.players_list.iter().map(
            |p| JsonValue::String(String::from(p))).collect();
        mineonline_json.insert("players",
                               JsonValue::Array(players));
        mineonline_json.insert("max",
                               JsonValue::Number(Number::from(self.max_users)));
        mineonline_json.insert("name",
                               JsonValue::String(String::from(&self.name)));
        mineonline_json.insert("onlinemode",
                               JsonValue::String(String::from(&self.online)));
        mineonline_json.insert("md5",
                               JsonValue::String(String::from(&self.client_hash)));
        mineonline_json.insert("whitelisted",
                               JsonValue::Boolean(self.whitelisted));
        mineonline_json.insert("whitelistUsers", JsonValue::Array(
            self.whitelisted_users.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));
        mineonline_json.insert("whitelistIPs", JsonValue::Array(
            self.whitelisted_ips.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));
        mineonline_json.insert("whitelistUUIDs", JsonValue::Array(
            self.whitelisted_uuids.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));
        mineonline_json.insert("bannedUsers", JsonValue::Array(
            self.banned_users.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));
        mineonline_json.insert("bannedIPs", JsonValue::Array(
            self.banned_ips.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));
        mineonline_json.insert("bannedUUIDs", JsonValue::Array(
            self.banned_uuids.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));
        // TODO: Support Players List
        // TODO: Support Owner Name

        let stringified = stringify(mineonline_json);
        self.request = (&stringified).parse().unwrap();
        stringified

    }

    pub fn get_user_count(&self) -> u16 {
        self.users
    }

    pub fn get_whitelist(&self) -> (&Vec<String>, &Vec<String>) {
        (&self.whitelisted_users, &self.whitelisted_ips)
    }

    pub fn get_request(&self) -> &str {
        &self.request
    }
    /// Causes a heartbeat request to be made to the server
    pub async fn beat(&mut self) {
        let request_client = reqwest::Client::new();
        let request = request_client.post(Url::parse(&self.url)
            .expect("Failed ot parse to URL")
        ).header("content-type", "application/json").body(self.request.clone());
        // println!("Request: {:?}", request);
        let response = request.send().await.expect("Failed to make post request");
        // println!("Response: {:?}", response);
        if response.status() != StatusCode::OK {
            panic!("Heartbeat Request Failed: {}", response.status());
        }
    }
}