use colored::Colorize;

pub fn fail(error: &str) -> String {
    error.red().to_string()
}

pub fn fail_symbol() -> String {
    "⍨".red().to_string()
}

pub fn pass(info: &str) -> String {
    info.green().to_string()
}

pub fn pass_symbol() -> String {
    "✓".green().to_string()
}

pub fn warn(warning: &str) -> String {
    warning.yellow().to_string()
}

pub fn warn_symbol() -> String {
    "?".yellow().to_string()
}
