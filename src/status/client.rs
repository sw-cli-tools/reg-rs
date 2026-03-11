use std::net;

/// provide details to start a web browser
pub fn start(port: u16) {
    log::info!("client/start");
    log::debug!("client/start port: {}", port);
    println!("open: http://{}:{}/status", net::Ipv4Addr::LOCALHOST, port);
}
