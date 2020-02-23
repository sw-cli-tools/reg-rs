use std::net;

use log;

pub fn start(port: u16) {
    log::info!("client/start");
    md!(port);
    println!("open: http://{}:{}/status", net::Ipv4Addr::LOCALHOST.to_string(), port);
}
