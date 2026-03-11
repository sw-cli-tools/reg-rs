use colored::Colorize;

/// color an error string red
pub fn fail(error: &str) -> String {
    error.red().to_string()
}

/// return a red failure symbol
pub fn fail_symbol() -> String {
    "⍨".red().to_string()
}

/// color a pass string green
pub fn pass(info: &str) -> String {
    info.green().to_string()
}

/// return a green pass symbol
pub fn pass_symbol() -> String {
    "✓".green().to_string()
}

/// color a warning string yellow
pub fn warn(warning: &str) -> String {
    warning.yellow().to_string()
}

/// return a yellow warning symbol
pub fn warn_symbol() -> String {
    "?".yellow().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_contains_input() {
        let result = fail("error");
        assert!(result.contains("error"));
    }

    #[test]
    fn test_pass_contains_input() {
        let result = pass("ok");
        assert!(result.contains("ok"));
    }

    #[test]
    fn test_warn_contains_input() {
        let result = warn("caution");
        assert!(result.contains("caution"));
    }

    #[test]
    fn test_fail_symbol_not_empty() {
        assert!(!fail_symbol().is_empty());
    }

    #[test]
    fn test_pass_symbol_not_empty() {
        assert!(!pass_symbol().is_empty());
    }

    #[test]
    fn test_warn_symbol_not_empty() {
        assert!(!warn_symbol().is_empty());
    }
}
