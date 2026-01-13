/// Memory-efficient issue caching to prevent memory spikes from large descriptions
use crate::beads::models::Issue;
use std::collections::HashMap;

/// Maximum size for cached descriptions (100KB)
const MAX_CACHED_DESCRIPTION_SIZE: usize = 100 * 1024;

/// Maximum size for truncated descriptions in list view (1KB)
const TRUNCATED_DESCRIPTION_SIZE: usize = 1024;

/// Issue cache that truncates large descriptions to prevent memory spikes
pub struct IssueCache {
    /// Full descriptions for currently viewed issues
    full_descriptions: HashMap<String, String>,
    /// Memory usage tracking
    total_description_memory: usize,
}

impl IssueCache {
    pub fn new() -> Self {
        Self {
            full_descriptions: HashMap::new(),
            total_description_memory: 0,
        }
    }

    /// Process issues for list view - truncate large descriptions
    pub fn prepare_for_list_view(&mut self, issues: &mut [Issue]) {
        for issue in issues.iter_mut() {
            if let Some(ref desc) = issue.description {
                if desc.len() > TRUNCATED_DESCRIPTION_SIZE {
                    // Store full description in cache if not too large
                    if desc.len() <= MAX_CACHED_DESCRIPTION_SIZE {
                        self.full_descriptions
                            .insert(issue.id.clone(), desc.clone());
                    }

                    // Truncate description for list view
                    let truncated = format!(
                        "{}... [{} more chars]",
                        &desc[..TRUNCATED_DESCRIPTION_SIZE.min(desc.len())],
                        desc.len() - TRUNCATED_DESCRIPTION_SIZE
                    );
                    issue.description = Some(truncated);
                }
            }
        }

        // Update memory tracking
        self.update_memory_usage();
    }

    /// Get full description for an issue (from cache or issue)
    pub fn get_full_description(&self, issue: &Issue) -> Option<String> {
        // Check if we have the full version cached
        if let Some(cached) = self.full_descriptions.get(&issue.id) {
            return Some(cached.clone());
        }

        // Return the current description (might be truncated)
        issue.description.clone()
    }

    /// Clear cache for an issue (e.g., when issue is updated)
    pub fn invalidate(&mut self, issue_id: &str) {
        self.full_descriptions.remove(issue_id);
        self.update_memory_usage();
    }

    /// Clear all cached descriptions
    pub fn clear(&mut self) {
        self.full_descriptions.clear();
        self.total_description_memory = 0;
    }

    /// Get total memory used by cached descriptions
    pub fn memory_usage(&self) -> usize {
        self.total_description_memory
    }

    /// Update memory usage tracking
    fn update_memory_usage(&mut self) {
        self.total_description_memory = self
            .full_descriptions
            .values()
            .map(|desc| desc.len())
            .sum();
    }

    /// Get cache statistics
    pub fn stats(&self) -> IssueCacheStats {
        IssueCacheStats {
            cached_count: self.full_descriptions.len(),
            total_memory_bytes: self.total_description_memory,
            total_memory_mb: self.total_description_memory as f64 / (1024.0 * 1024.0),
        }
    }
}

impl Default for IssueCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the issue cache
#[derive(Debug, Clone)]
pub struct IssueCacheStats {
    pub cached_count: usize,
    pub total_memory_bytes: usize,
    pub total_memory_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::models::{IssueStatus, IssueType, Priority};
    use chrono::Utc;

    fn create_test_issue(id: &str, description: Option<String>) -> Issue {
        Issue {
            id: id.to_string(),
            title: "Test Issue".to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type: IssueType::Task,
            description,
            assignee: None,
            labels: vec![],
            dependencies: vec![],
            blocks: vec![],
            created: Utc::now(),
            updated: Utc::now(),
            closed: None,
            notes: vec![],
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache = IssueCache::new();
        assert_eq!(cache.memory_usage(), 0);
        assert_eq!(cache.stats().cached_count, 0);
    }

    #[test]
    fn test_small_descriptions_not_truncated() {
        let mut cache = IssueCache::new();
        let small_desc = "Small description".to_string();
        let mut issues = vec![create_test_issue("test-1", Some(small_desc.clone()))];

        cache.prepare_for_list_view(&mut issues);

        assert_eq!(issues[0].description, Some(small_desc));
        assert_eq!(cache.stats().cached_count, 0); // Small descriptions not cached
    }

    #[test]
    fn test_large_descriptions_truncated() {
        let mut cache = IssueCache::new();
        let large_desc = "x".repeat(2000); // 2KB description
        let mut issues = vec![create_test_issue("test-1", Some(large_desc.clone()))];

        cache.prepare_for_list_view(&mut issues);

        // Description should be truncated
        let truncated = issues[0].description.as_ref().unwrap();
        assert!(truncated.len() < large_desc.len());
        assert!(truncated.contains("more chars"));

        // Full description should be cached
        assert_eq!(cache.stats().cached_count, 1);
        let full = cache.get_full_description(&issues[0]).unwrap();
        assert_eq!(full, large_desc);
    }

    #[test]
    fn test_very_large_descriptions_not_cached() {
        let mut cache = IssueCache::new();
        // Create description larger than MAX_CACHED_DESCRIPTION_SIZE
        let very_large_desc = "x".repeat(150 * 1024); // 150KB
        let mut issues = vec![create_test_issue("test-1", Some(very_large_desc.clone()))];

        cache.prepare_for_list_view(&mut issues);

        // Description should be truncated
        assert!(issues[0].description.as_ref().unwrap().len() < very_large_desc.len());

        // But NOT cached (too large)
        assert_eq!(cache.stats().cached_count, 0);
    }

    #[test]
    fn test_multiple_issues() {
        let mut cache = IssueCache::new();
        let mut issues = vec![
            create_test_issue("test-1", Some("x".repeat(2000))),
            create_test_issue("test-2", Some("y".repeat(3000))),
            create_test_issue("test-3", Some("Small".to_string())),
        ];

        cache.prepare_for_list_view(&mut issues);

        // Two large descriptions should be cached
        assert_eq!(cache.stats().cached_count, 2);
        assert!(cache.memory_usage() > 0);
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = IssueCache::new();
        let mut issues = vec![create_test_issue("test-1", Some("x".repeat(2000)))];

        cache.prepare_for_list_view(&mut issues);
        assert_eq!(cache.stats().cached_count, 1);

        cache.invalidate("test-1");
        assert_eq!(cache.stats().cached_count, 0);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = IssueCache::new();
        let mut issues = vec![
            create_test_issue("test-1", Some("x".repeat(2000))),
            create_test_issue("test-2", Some("y".repeat(3000))),
        ];

        cache.prepare_for_list_view(&mut issues);
        assert_eq!(cache.stats().cached_count, 2);

        cache.clear();
        assert_eq!(cache.stats().cached_count, 0);
        assert_eq!(cache.memory_usage(), 0);
    }

    #[test]
    fn test_memory_usage_calculation() {
        let mut cache = IssueCache::new();
        let desc1 = "x".repeat(2000);
        let desc2 = "y".repeat(3000);
        let mut issues = vec![
            create_test_issue("test-1", Some(desc1.clone())),
            create_test_issue("test-2", Some(desc2.clone())),
        ];

        cache.prepare_for_list_view(&mut issues);

        let expected_memory = desc1.len() + desc2.len();
        assert_eq!(cache.memory_usage(), expected_memory);
    }
}
