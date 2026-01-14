/// Error types for beads-rs library
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BeadsError>;

#[derive(Error, Debug)]
pub enum BeadsError {
    #[error("IO error: {0}\nCheck file permissions and disk space. Run 'bd doctor' to diagnose issues.")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0} (Input: {1})\nThe issue database may be corrupted. Try 'bd sync' to restore from remote, or check .beads/issues.jsonl format.")]
    Json(serde_json::Error, String),

    #[error("Command execution error: {0}\nTry running 'bd doctor' to diagnose issues.")]
    CommandError(String),

    #[error("Timeout: operation took longer than {0}ms\nTry increasing timeout with --timeout={0} or check system resources.")]
    Timeout(u64),

    #[error("Cancelled: {0}")]
    Cancelled(String),

    #[error("Invalid issue ID: {0}\nIssue IDs must be in format 'beads-xxxx-xxxx' (e.g., beads-tui-a1b2).")]
    InvalidIssueId(String),

    #[error("Issue not found: {0}\nRun 'bd list' to see all issues or 'bd show {0}' for details.")]
    IssueNotFound(String),

    #[error("Invalid configuration: {0}\nCheck your .beads/config file or run 'bd init' to reinitialize.")]
    InvalidConfig(String),

    #[error("Beads CLI not found in PATH\nInstall beads with: cargo install beads\nOr ensure 'bd' is in your system PATH.")]
    BeadsNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_error_display() {
        let err = BeadsError::CommandError("command failed".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Command execution error: command failed"));
        assert!(msg.contains("bd doctor"));
    }

    #[test]
    fn test_timeout_error_display() {
        let err = BeadsError::Timeout(5000);
        let msg = err.to_string();
        assert!(msg.contains("Timeout"));
        assert!(msg.contains("5000ms"));
        assert!(msg.contains("timeout") || msg.contains("resources"));
    }

    #[test]
    fn test_cancelled_error_display() {
        let err = BeadsError::Cancelled("user requested".to_string());
        assert_eq!(err.to_string(), "Cancelled: user requested");
    }

    #[test]
    fn test_invalid_issue_id_display() {
        let err = BeadsError::InvalidIssueId("bad-id".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid issue ID: bad-id"));
        assert!(msg.contains("beads-xxxx-xxxx"));
    }

    #[test]
    fn test_issue_not_found_display() {
        let err = BeadsError::IssueNotFound("beads-123".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Issue not found: beads-123"));
        assert!(msg.contains("bd list") || msg.contains("bd show"));
    }

    #[test]
    fn test_invalid_config_display() {
        let err = BeadsError::InvalidConfig("missing field".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid configuration: missing field"));
        assert!(msg.contains("config") || msg.contains("bd init"));
    }

    #[test]
    fn test_beads_not_found_display() {
        let err = BeadsError::BeadsNotFound;
        let msg = err.to_string();
        assert!(msg.contains("Beads CLI not found"));
        assert!(msg.contains("cargo install beads") || msg.contains("PATH"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let beads_err: BeadsError = io_err.into();
        let msg = beads_err.to_string();
        assert!(msg.contains("IO error"));
        assert!(msg.contains("file not found"));
        assert!(msg.contains("permissions") || msg.contains("bd doctor"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{invalid json";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let beads_err = BeadsError::Json(json_err, json_str.to_string());
        let msg = beads_err.to_string();
        assert!(msg.contains("JSON parsing error"));
        assert!(msg.contains("{invalid json"));
        assert!(msg.contains("corrupted") || msg.contains("bd sync"));
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
            let msg = e.to_string();
            assert!(msg.contains("Beads CLI not found"));
            assert!(msg.contains("cargo install beads") || msg.contains("PATH"));
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
        assert!(errors[1].to_string().contains("Timeout"));
        assert!(
            errors[2].to_string().contains("not found") || errors[2].to_string().contains("CLI")
        );
    }
}
