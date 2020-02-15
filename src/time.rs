use chrono::Local;

pub fn now() -> String {
    let date = Local::now();
    format!("{}", date.format("%Y-%m-%dT%H:%M:%S"))
}
