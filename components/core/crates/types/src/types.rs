/// Test Results data
#[derive(Debug)]
pub struct TestResults {
    /// Test Name
    pub name: String,
    /// Subject command
    pub command: String,
    /// test results creation time
    pub time_created: String,
    /// test exit code
    pub exit_code: i32,
    /// test captured stderr
    pub stderr: String,
    /// test captured stdout
    pub stdout: String,
}

/// Test result regression types
#[derive(Clone, Copy)]
pub enum RegressionType {
    /// The latest test result exit code
    ActualCode = 1,
    /// The reference test result exit code
    ExpectedCode,
    /// The latest test result stderr additions
    StderrAdd,
    /// The latest test result stderr removals
    StderrRemove,
    /// The latest test result stderr matches
    StderrSame,
    /// The latest test result stdout additions
    StdoutAdd,
    /// The latest test result stdout removals
    StdoutRemove,
    /// The latest test result stdout matches
    StdoutSame,
}

impl RegressionType {
    /// Parse a type code string into a RegressionType variant.
    /// Returns None for unknown codes.
    pub fn from_code(type_code: &str) -> Option<Self> {
        let code: u8 = type_code.parse().ok()?;
        match code {
            1 => Some(Self::ActualCode),
            2 => Some(Self::ExpectedCode),
            3 => Some(Self::StderrAdd),
            4 => Some(Self::StderrRemove),
            5 => Some(Self::StderrSame),
            6 => Some(Self::StdoutAdd),
            7 => Some(Self::StdoutRemove),
            8 => Some(Self::StdoutSame),
            _ => None,
        }
    }

    /// Convert a stored type code string to a human-readable label for display.
    /// Returns None for "same" types (5, 8) and unknown codes.
    pub fn display_label(type_code: &str) -> Option<&'static str> {
        match Self::from_code(type_code)? {
            Self::ActualCode => Some("Actual exit code"),
            Self::ExpectedCode => Some("Expected exit code"),
            Self::StderrAdd => Some("stderr add"),
            Self::StderrRemove => Some("stderr remove"),
            Self::StdoutAdd => Some("stdout add"),
            Self::StdoutRemove => Some("stdout remove"),
            Self::StderrSame | Self::StdoutSame => None,
        }
    }

    /// Check if a type code represents a "has differences" type for counting.
    pub fn has_differences(type_code: &str) -> bool {
        matches!(
            Self::from_code(type_code),
            Some(
                Self::ActualCode
                    | Self::StderrAdd
                    | Self::StderrRemove
                    | Self::StdoutAdd
                    | Self::StdoutRemove
            )
        )
    }
}
