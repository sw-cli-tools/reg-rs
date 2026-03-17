/// File system watcher setup utilities
use std::path::PathBuf;

use crate::error::RegError;

/// Extract pattern and data dir from app state
pub fn extract_watch_info(
    app_state: &crate::status::AppState,
) -> Result<(String, PathBuf), RegError> {
    let state_data = app_state
        .state_data
        .lock()
        .map_err(|e| RegError::MutexPoisoned(format!("monitor startup: {}", e)))?;
    Ok((state_data.pattern.clone(), state_data.data_dir.clone()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_watch_info_returns_pattern_and_dir() {}
}
