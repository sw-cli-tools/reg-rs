use colored::Colorize;

pub fn fail(error: &str) -> String {
    error.red().to_string()
}

pub fn pass(error: &str) -> String {
    error.green().to_string()
}

pub fn pass_symbol() -> String {
    "✓".green().to_string()
}

pub fn warn(warning: &str) -> String {
    warning.yellow().to_string()
}
