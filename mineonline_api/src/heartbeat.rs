//! # Heartbeat
//! Used for sending Minecraft Heartbeats to [Mineonline](http://mineonline.codie.gg)

use rand::{thread_rng, Rng};
use json::{parse, stringify, JsonValue};
use json::number::Number;
use json::object::Object;
use reqwest::{Body, Url, StatusCode};
use std::time::Instant;
use std::borrow::Borrow;
use std::thread;

const MINEONLINE_HEARTBEAT_URL: &str = "http://mineonline.codie.gg/mineonline/listserver.jsp";

/// Heartbeat Object
pub struct Heartbeat {
    ip: String,
    port: u16,
    name: String,
    public: String,
    max_players: u16,
    online: String,
    client_hash: String,
    users: u16,
    whitelisted: bool,
    whitelisted_users: Vec<String>,
    whitelisted_ips: Vec<String>,
    banned_users: Vec<String>,
    banned_ips: Vec<String>,
    mineonline_req: String,
}

impl Heartbeat {
    /// Create a Heartbeat Object.
    ///
    pub fn new(ip: &str, port: u16, name: &str, public: bool, max_players: u16, online: bool,
               client_hash: &str, whitelisted: bool) -> Self {
        Self {
            ip: ip.to_string(),
            port,
            name: name.to_string(),
            public: public.to_string(),
            max_players,
            online: online.to_string(),
            client_hash: client_hash.to_string(),
            users: 0,
            whitelisted,
            whitelisted_users: vec![],
            whitelisted_ips: vec![],
            banned_users: vec![],
            banned_ips: vec![],
            mineonline_req: "".to_string(),
        }
    }
    /// Update the number of users currently connected to the server in the heartbeat.
    pub fn update_users(&mut self, user_count: u16) {
        self.users = user_count;
    }

    /// Update the servers ban list in the heartbeat.
    pub fn update_bans(&mut self, banned_users: Vec<String>, banned_ips: Vec<String>) {
        self.banned_users = banned_users;
        self.banned_ips = banned_ips;
    }
    /// Update the servers whitelist in the heartbeat.
    pub fn update_whitelist(&mut self, wl_users: Vec<String>, wl_ips: Vec<String>) {
        self.whitelisted_users = wl_users;
        self.whitelisted_ips = wl_ips;
    }
    /// Builds the request data from the heartbeat.
    pub fn build_mineonline_request(&mut self) -> String {
        let mut mineonline_json: Object = Object::new();
        mineonline_json.insert("ip",
                               JsonValue::String(String::from(&self.ip)));
        mineonline_json.insert("port",
                               JsonValue::String(String::from(&self.port.to_string())));
        mineonline_json.insert("users",
                               JsonValue::Number(Number::from(self.users)));
        mineonline_json.insert("max",
                               JsonValue::Number(Number::from(self.max_players)));
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
        mineonline_json.insert("bannedUsers", JsonValue::Array(
            self.banned_users.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));
        mineonline_json.insert("bannedIPs", JsonValue::Array(
            self.banned_ips.iter()
                .map(|x| JsonValue::String(String::from(x))).collect()));

        let stringified = stringify(mineonline_json);
        self.mineonline_req = (&stringified).parse().unwrap();
        stringified

    }

    pub fn get_user_count(&self) -> u16 {
        self.users
    }

    pub fn get_whitelist(&self) -> (&Vec<String>, &Vec<String>) {
        (&self.whitelisted_users, &self.whitelisted_ips)
    }

    pub fn get_request(&self) -> &str {
        &self.mineonline_req
    }
    /// Causes a heartbeat request to be made to the server
    pub async fn beat(&mut self) {
        let request_client = reqwest::Client::new();
        let request = request_client.post(Url::parse(&MINEONLINE_HEARTBEAT_URL)
            .expect("Failed ot parse to URL")
        ).header("content-type", "application/json").body(self.mineonline_req.clone());
        // println!("Request: {:?}", request);
        let response = request.send().await.expect("Failed to make post request");
        // println!("Response: {:?}", response);
        if response.status() != StatusCode::OK {
            panic!("Heartbeat Request Failed: {}", response.status());
        }
    }
}