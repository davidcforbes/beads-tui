/// Error types for beads-rs library
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BeadsError>;

#[derive(Error, Debug)]
pub enum BeadsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0} (Input: {1})")]
    Json(serde_json::Error, String),

    #[error("Command execution error: {0}")]
    CommandError(String),

    #[error("Timeout error: operation took longer than {0}ms")]
    Timeout(u64),

    #[error("Cancelled: {0}")]
    Cancelled(String),

    #[error("Invalid issue ID: {0}")]
    InvalidIssueId(String),

    #[error("Issue not found: {0}")]
    IssueNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Beads CLI not found in PATH")]
    BeadsNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_error_display() {
        let err = BeadsError::CommandError("command failed".to_string());
        assert_eq!(err.to_string(), "Command execution error: command failed");
    }

    #[test]
    fn test_timeout_error_display() {
        let err = BeadsError::Timeout(5000);
        assert_eq!(
            err.to_string(),
            "Timeout error: operation took longer than 5000ms"
        );
    }

    #[test]
    fn test_cancelled_error_display() {
        let err = BeadsError::Cancelled("user requested".to_string());
        assert_eq!(err.to_string(), "Cancelled: user requested");
    }

    #[test]
    fn test_invalid_issue_id_display() {
        let err = BeadsError::InvalidIssueId("bad-id".to_string());
        assert_eq!(err.to_string(), "Invalid issue ID: bad-id");
    }

    #[test]
    fn test_issue_not_found_display() {
        let err = BeadsError::IssueNotFound("beads-123".to_string());
        assert_eq!(err.to_string(), "Issue not found: beads-123");
    }

    #[test]
    fn test_invalid_config_display() {
        let err = BeadsError::InvalidConfig("missing field".to_string());
        assert_eq!(err.to_string(), "Invalid configuration: missing field");
    }

    #[test]
    fn test_beads_not_found_display() {
        let err = BeadsError::BeadsNotFound;
        assert_eq!(err.to_string(), "Beads CLI not found in PATH");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let beads_err: BeadsError = io_err.into();
        assert!(beads_err.to_string().contains("IO error"));
        assert!(beads_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{invalid json";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let beads_err = BeadsError::Json(json_err, json_str.to_string());
        assert!(beads_err.to_string().contains("JSON parsing error"));
        assert!(beads_err.to_string().contains("{invalid json"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, 42);
        }
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(BeadsError::BeadsNotFound);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Beads CLI not found in PATH");
        }
    }

    #[test]
    fn test_error_debug_format() {
        let err = BeadsError::Timeout(1000);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Timeout"));
        assert!(debug_str.contains("1000"));
    }

    #[test]
    fn test_multiple_error_types() {
        let errors = [
            BeadsError::CommandError("test".to_string()),
            BeadsError::Timeout(100),
            BeadsError::BeadsNotFound,
        ];

        assert_eq!(errors.len(), 3);
        assert!(errors[0].to_string().contains("Command execution error"));
        assert!(errors[1].to_string().contains("Timeout error"));
        assert!(errors[2].to_string().contains("not found"));
    }
}
