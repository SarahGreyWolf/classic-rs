use mineonline_api::heartbeat::Heartbeat;
use std::borrow::Borrow;

#[test]
fn create_request() {
    let mut heartbeat = Heartbeat::new(
        "0.0.0.0",
        25565,
        "TestServer",
        true,
        20,
        true,
        "0",
        true
    );
    heartbeat.update_whitelist(vec!["SarahGreyWolf".to_string()],
                               vec!["192.168.0.14".to_string()]);
    heartbeat.update_bans(vec!["SarahGreyWolf".to_string()],
                          vec!["192.168.0.14".to_string()]);
    heartbeat.update_users(2);
    let mineonline_json = "{\"ip\":\"0.0.0.0\",\
        \"port\":\"25565\",\
        \"users\":2,\
        \"max\":20,\
        \"name\":\"TestServer\",\
        \"onlinemode\":\"true\",\
        \"md5\":\"0\",\
        \"whitelisted\":true,\
        \"whitelistUsers\":[\"SarahGreyWolf\"],\
        \"whitelistIPs\":[\"192.168.0.14\"],\
        \"bannedUsers\":[\"SarahGreyWolf\"],\
        \"bannedIPs\":[\"192.168.0.14\"]}";


    assert_eq!(heartbeat.build_mineonline_request(), mineonline_json);
}

#[test]
fn update_users() {
    let mut heartbeat = Heartbeat::new(
        "0.0.0.0",
        25565,
        "TestServer",
        true,
        20,
        true,
        "0",
        true
    );

    assert_eq!(heartbeat.get_user_count(), 0);
    heartbeat.update_users(5);
    assert_eq!(heartbeat.get_user_count(), 5);
}

#[test]
fn update_whitelist_user() {
    let mut heartbeat = Heartbeat::new(
        "0.0.0.0",
        25565,
        "TestServer",
        true,
        20,
        true,
        "0",
        true
    );
    let mut whitelisted_users: Vec<String> = Vec::new();
    assert_eq!(heartbeat.get_whitelist().0, &whitelisted_users);
    whitelisted_users.push("SarahGreyWolf".to_string());
    heartbeat.update_whitelist(vec!["SarahGreyWolf".to_string()],Vec::new());
    assert_eq!(heartbeat.get_whitelist().0, &whitelisted_users);
}

#[test]
fn update_whitelist_ip() {
    let mut heartbeat = Heartbeat::new(
        "0.0.0.0",
        25565,
        "TestServer",
        true,
        20,
        true,
        "0",
        true
    );
    let mut whitelisted_ips: Vec<String> = Vec::new();
    assert_eq!(heartbeat.get_whitelist().1, &whitelisted_ips);
    whitelisted_ips.push("127.0.0.1".to_string());
    heartbeat.update_whitelist(vec!["127.0.0.1".to_string()],Vec::new());
    assert_eq!(heartbeat.get_whitelist().1, &whitelisted_ips);
}