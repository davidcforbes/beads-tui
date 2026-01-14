//! Global tokio runtime for async operations
//!
//! This module provides a single shared tokio runtime for all async operations
//! in the application, avoiding the anti-pattern of creating a new runtime
//! for each operation.

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

/// Global tokio runtime for async operations.
///
/// Creating a new runtime for each async operation is an anti-pattern that:
/// - Adds 5-50ms overhead per operation
/// - Wastes memory by creating duplicate thread pools
/// - Can cause crashes if runtime creation fails
///
/// Instead, we use a single shared runtime for all async operations.
pub static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create tokio runtime"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_available() {
        // Verify the runtime can be accessed
        RUNTIME.block_on(async {
            // Simple async operation
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        });
    }

    #[test]
    fn test_runtime_reusable() {
        // Verify the same runtime is used across multiple calls
        let result1 = RUNTIME.block_on(async { 42 });
        let result2 = RUNTIME.block_on(async { 100 });
        assert_eq!(result1, 42);
        assert_eq!(result2, 100);
    }
}
