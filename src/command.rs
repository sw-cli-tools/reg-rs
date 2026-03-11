use crate::config;
use crate::db;
use crate::finder;
use crate::queries;
use crate::reporters::generate_reports;
use crate::runner;
use crate::status;

/// Create a test result.
///
/// If the test path is just a filename (no directory separators), it is
/// placed in the default data directory (`~/.local/reg-rs/`). The `.tdb`
/// extension is appended automatically if missing.
#[allow(clippy::collapsible_if)]
pub fn create_original(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/create_original");
    md!(&config);
    if let Some((test, command)) = config.extract_test_and_command() {
        let db_name = resolve_test_path(&test);
        if let Some(test_result) = runner::run_one(&db_name, &command, false)? {
            db::reset_differences(&db_name)?;
            db::reset_latest_results(&db_name)?;
            db::store_results(
                &db_name,
                &test_result,
                queries::StatementContext::original(),
            )?;
        }
    }
    Ok(())
}

/// Resolve a test path: if it has no directory component, place it in the
/// data directory. Append `.tdb` extension if missing.
fn resolve_test_path(test: &str) -> String {
    let path = std::path::Path::new(test);
    let mut resolved = if path.parent().is_some_and(|p| p != std::path::Path::new("")) {
        // Has a directory component - use as-is
        path.to_path_buf()
    } else {
        // Just a filename - put it in the data directory
        crate::data_dir().join(test)
    };
    if resolved.extension().is_none_or(|ext| ext != "tdb") {
        let mut name = resolved.file_name().unwrap_or_default().to_os_string();
        name.push(".tdb");
        resolved.set_file_name(name);
    }
    resolved.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_bare_name() {
        let resolved = resolve_test_path("my_test");
        let data_dir = crate::data_dir();
        assert_eq!(resolved, data_dir.join("my_test.tdb").to_string_lossy());
    }

    #[test]
    fn test_resolve_bare_name_with_tdb() {
        let resolved = resolve_test_path("my_test.tdb");
        let data_dir = crate::data_dir();
        assert_eq!(resolved, data_dir.join("my_test.tdb").to_string_lossy());
    }

    #[test]
    fn test_resolve_path_with_directory() {
        let resolved = resolve_test_path("/tmp/tests/foo");
        assert_eq!(resolved, "/tmp/tests/foo.tdb");
    }

    #[test]
    fn test_resolve_path_with_directory_and_tdb() {
        let resolved = resolve_test_path("/tmp/tests/foo.tdb");
        assert_eq!(resolved, "/tmp/tests/foo.tdb");
    }

    #[test]
    fn test_resolve_relative_path_with_directory() {
        let resolved = resolve_test_path("subdir/foo");
        assert_eq!(resolved, "subdir/foo.tdb");
    }
}

/// Update a test result
pub fn update_latest(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/update_latest");
    runner::run_many(config)?;
    Ok(())
}

/// remove test results
pub fn remove_all(config: &config::Config) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/remove_all");
    let tests = finder::discover(config.extract_pattern().to_string())?;
    md!(&tests);
    for test in tests.found {
        db::drop_all_results(&test)?;
    }
    log::info!("command/remove_all done");
    Ok(())
}

/// report latest test results
pub fn report_latest(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/report_latest");
    generate_reports(config)?;
    log::info!("command/report_latest done");
    Ok(())
}

/// start status client and server
pub async fn status_server(
    config: &config::Config,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    log::info!("command/status_server");
    status::start_client(config)?;
    status::start_server(config).await?; // loops
    Ok(())
}
