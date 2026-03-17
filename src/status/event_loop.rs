//! Event loop utilities for file system monitoring

/// Process a single file system event
pub fn process_event(event: notify::DebouncedEvent) {
    log::debug!("monitor/watch event: {:?}", event);
}

/// Update shared state with current timestamp
pub fn update_timestamp(state_data: &mut crate::status::StateData) {
    state_data.state_updated = crate::time::now();
    log::info!(
        "monitor/watch state_data.state_updated: {}",
        &state_data.state_updated
    );
    log::debug!("monitor state_updated: {}", &state_data.state_updated);
}

/// Initialize the monitor loop index
pub fn init_loop_index() -> usize {
    0
}

/// Increment loop index and check if we should continue
pub fn increment_and_continue(index: usize) -> usize {
    index.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_loop_index_returns_zero() {
        assert_eq!(init_loop_index(), 0);
    }

    #[test]
    fn test_increment_and_continue_increments() {
        assert_eq!(increment_and_continue(0), 1);
        assert_eq!(increment_and_continue(10), 11);
    }
}
