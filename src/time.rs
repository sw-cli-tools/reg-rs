use chrono::Local;

/// current time as formatted string
pub fn now() -> String {
    let date = Local::now();
    format!("{}", date.format("%Y-%m-%dT%H:%M:%S"))
}
