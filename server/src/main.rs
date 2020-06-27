use tokio::prelude::*;

use mineonline_api::heartbeat::Heartbeat;
use mc_packets::Packet;
use mc_packets::classic::{ClientBound, ServerBound};

#[tokio::main]
async fn main() {
    let mut hearbeat: Heartbeat =
        Heartbeat::new(
            "192.168.0.14",
            25565,
            "Sarah's Pipe Dream",
            false,
            8,
            true,
            "90632803F45C15164587256A08C0ECB4",
            true
        );

    hearbeat.update_whitelist(vec!["SarahGreyWolf".to_string()], vec![]);
    hearbeat.build_mineonline_request();
    hearbeat.beat().await;
}
