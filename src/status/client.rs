pub fn start(port: u16) {
    md!(port);
    println!("open: http://localhost:{}/status", port);
}
