//! # Heartbeat
//! Used for sending Mojang Minecraft Heartbeats

use rand::{thread_rng, Rng};
use reqwest::{Body, Url, StatusCode};

/// Heartbeat Object
pub struct Heartbeat {
    url: String,
    ip: String,
    port: u16,
    name: String,
    public: String,
    max_users: u16,
    online: String,
    protocol: u16,
    salt: String,
    users: u16,
    request: Vec<(String, String)>,
}

impl Heartbeat {
    /// Create a Heartbeat Object.
    ///
    pub fn new(url: &str, ip: &str, port: u16, name: &str, public: bool, max_players: u16, online: bool,
               salt: &str, protocol: u16) -> Self {
        Self {
            url: url.to_string(),
            ip: ip.to_string(),
            port,
            name: name.to_string(),
            public: public.to_string(),
            max_users: max_players,
            online: online.to_string(),
            protocol,
            salt: salt.to_string(),
            users: 0,
            request: vec![],
        }
    }
    /// Update the number of users currently connected to the server in the heartbeat.
    pub fn update_users(&mut self, user_count: u16) {
        self.users = user_count;
    }
    /// Builds the request data from the heartbeat.
    pub fn build_request(&mut self) -> Vec<(String, String)> {
        let mut query: Vec<(String, String)> = vec![];
        query.push(("ip".to_string(), self.ip.to_string()));
        query.push(("port".to_string(), self.port.to_string()));
        query.push(("users".to_string(), self.users.to_string()));
        query.push(("max".to_string(), self.max_users.to_string()));
        query.push(("name".to_string(), self.name.to_string()));
        query.push(("public".to_string(), self.public.to_string()));
        query.push(("version".to_string(), self.protocol.to_string()));
        query.push(("salt".to_string(), self.salt.to_string()));
        self.request = query.clone();
        query
    }

    pub fn get_user_count(&self) -> u16 {
        self.users
    }

    pub fn get_request(&self) -> Vec<(String, String)> {
        self.request.clone()
    }
    /// Causes a heartbeat request to be made to the server
    pub async fn beat(&mut self) {
        let request_client = reqwest::Client::new();
        let request = request_client.post(Url::parse(&self.url)
            .expect("Failed ot parse to URL")
        ).form(&self.request);
        // println!("Request: {:?}", request);
        let response = request.send().await.expect("Failed to make post request");
        // println!("Response: {:?}", response);
        if response.status() != StatusCode::OK {
            panic!("Heartbeat Request Failed: {}", response.status());
        }
    }
}