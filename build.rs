use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");
    let version = format!("0.1.0-dev-{}", chrono::Local::now()).to_string(); // build time
    let version_fn = "pub fn generated_version() -> &'static str {
\"".to_string() + &version
        + &"\"
}";
    fs::write(&dest_path, version_fn).unwrap();
}
