use reg_rs_config::config::Config;
use reg_rs_types::error::Result;

/// Output test names for shell completion
pub fn complete(config: &Config) -> Result<()> {
    let pattern = config.extract_pattern().to_string();
    let tests = reg_rs_discover::finder::discover(pattern)?;
    for test_path in &tests.found {
        let name = format_test_name(test_path);
        println!("{}", name);
    }
    Ok(())
}

/// Format a test path as a display name (file stem only).
fn format_test_name(test_path: &str) -> String {
    std::path::Path::new(test_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}
