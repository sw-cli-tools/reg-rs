use log;

pub fn start(port: u16) {
    log::info!("client/start");
    md!(port);
    println!("open: http://localhost:{}/status", port);
}
