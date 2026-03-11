use chrono::Local;

/// current time as formatted string
pub fn now() -> String {
    let date = Local::now();
    format!("{}", date.format("%Y-%m-%dT%H:%M:%S"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_format() {
        let result = now();
        // Should match YYYY-MM-DDTHH:MM:SS
        assert_eq!(result.len(), 19, "expected 19 chars: {}", result);
        assert_eq!(&result[4..5], "-");
        assert_eq!(&result[7..8], "-");
        assert_eq!(&result[10..11], "T");
        assert_eq!(&result[13..14], ":");
        assert_eq!(&result[16..17], ":");
    }

    #[test]
    fn test_now_returns_different_from_empty() {
        let result = now();
        assert!(!result.is_empty());
    }
}
