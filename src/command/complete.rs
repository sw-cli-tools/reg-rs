use crate::config;
use crate::finder;

use super::utils::format_test_name;

/// Output test names for shell completion
pub fn complete(config: &config::Config) -> crate::error::Result<()> {
    let pattern = config.extract_pattern().to_string();
    let tests = finder::discover(pattern)?;
    for test_path in &tests.found {
        let name = format_test_name(test_path);
        println!("{}", name);
    }
    Ok(())
}
