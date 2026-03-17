use colored::Colorize;

/// Color an error string red
pub fn fail(error: &str) -> String {
    error.red().to_string()
}

/// Return a red failure symbol
pub fn fail_symbol() -> String {
    "⍨".red().to_string()
}

/// Color a pass string green
pub fn pass(info: &str) -> String {
    info.green().to_string()
}

/// Return a green pass symbol
pub fn pass_symbol() -> String {
    "✓".green().to_string()
}

/// Color a warning string yellow
pub fn warn(warning: &str) -> String {
    warning.yellow().to_string()
}

/// Return a yellow warning symbol
pub fn warn_symbol() -> String {
    "?".yellow().to_string()
}
