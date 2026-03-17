use std::net;

/// Print URL for opening status page in browser
pub fn start(port: u16) {
    log::info!("client/start");
    log::debug!("client/start port: {}", port);
    println!("open: http://{}:{}/", net::Ipv4Addr::LOCALHOST, port);
}
