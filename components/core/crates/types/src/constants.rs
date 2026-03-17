/// Default status port for web
pub const DEFAULT_STATUS_PORT: u16 = 4740;
/// Status banner
pub const STATUS_BANNER: &str = "reg-rs Status Server";
/// Test database file extension
pub const TDB_EXTENSION: &str = "tdb";
/// Lock file extension (appended to .tdb path)
pub const LOCK_EXTENSION: &str = "lock";
/// File watcher debounce interval in seconds
pub const FILE_WATCH_DEBOUNCE_SECS: u64 = 5;
/// Template spacer for report alignment
pub const REQUIRED_BLANK: &str = " ";
/// Metadata key for the preprocess command in the test database
pub const PREPROCESS_KEY: &str = "preprocess";
/// Metadata key for the diff mode in the test database
pub const DIFF_MODE_KEY: &str = "diff_mode";
/// File extension for test specification files
pub const RGT_EXTENSION: &str = "rgt";
/// File extension for expected stdout baseline
pub const OUT_EXTENSION: &str = "out";
/// File extension for expected stderr baseline
pub const ERR_EXTENSION: &str = "err";
