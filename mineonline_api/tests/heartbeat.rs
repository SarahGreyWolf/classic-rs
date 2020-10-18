use mineonline_api::heartbeat::Heartbeat;


#[test]
fn create_request() {
    let mut heartbeat = Heartbeat::new(
        "",
        "0.0.0.0",
        25565,
        "TestServer",
        true,
        20,
        true,
        "0",
        true
    );
    heartbeat.update_users(2);
    let mineonline_json = "{\"ip\":\"0.0.0.0\",\
        \"port\":\"25565\",\
        \"users\":2,\
        \"max\":20,\
        \"name\":\"TestServer\",\
        \"onlinemode\":\"true\",\
        \"md5\":\"0\",\
        \"whitelisted\":true\"]}";


    assert_eq!(heartbeat.build_request(), mineonline_json);
}

#[test]
fn update_users() {
    let mut heartbeat = Heartbeat::new(
        "",
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