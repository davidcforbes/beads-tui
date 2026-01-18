/// Demo data generation for testing views without a real beads database
use crate::beads::models::{Issue, IssueFlags, IssueStatus, IssueType, Priority};
use crate::ui::views::{DatabaseStats, LabelStats};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// A complete demo dataset with issues, stats, and metadata
#[derive(Debug, Clone)]
pub struct DemoDataset {
    pub issues: Vec<Issue>,
    pub label_stats: Vec<LabelStats>,
    pub database_stats: DatabaseStats,
}

impl DemoDataset {
    /// Generate a demo dataset based on the specified type
    ///
    /// # Dataset Types
    /// - `small` - 15 issues, simple dependencies (good for screenshots)
    /// - `medium` - 50 issues, realistic workflows (integration testing)
    /// - `large` - 300 issues, stress testing (performance validation)
    /// - `deps` - 60 issues, complex dependency graphs (dependency view)
    /// - `edge` - 25 issues, edge cases (unicode, long text, special chars)
    pub fn generate(dataset_type: &str) -> Result<Self, String> {
        match dataset_type.to_lowercase().as_str() {
            "small" => Ok(Self::generate_small()),
            "medium" => Ok(Self::generate_medium()),
            "large" => Ok(Self::generate_large()),
            "deps" => Ok(Self::generate_deps()),
            "edge" => Ok(Self::generate_edge()),
            _ => Err(format!(
                "Unknown dataset type: {}. Valid types: small, medium, large, deps, edge",
                dataset_type
            )),
        }
    }

    /// Generate small dataset (15 issues)
    fn generate_small() -> Self {
        let mut generator = IssueGenerator::new(1000);
        let mut issues = Vec::new();

        // Create a simple project with epics, features, tasks, and bugs
        let epic1 = generator.epic("User Authentication System");
        let feat1 = generator.feature("Login Page", &["frontend", "auth"]);
        let feat2 = generator.feature("Registration Form", &["frontend", "auth"]);
        let task1 = generator.task("Design login UI mockups", &["design", "frontend"]);
        let task2 = generator.task("Implement JWT token generation", &["backend", "auth"]);
        let task3 = generator.task("Add password validation", &["backend"]);
        let bug1 = generator.bug("Login fails with special characters", &["frontend", "critical"]);
        let bug2 = generator.bug("Password reset email not sending", &["backend"]);

        // Set some relationships
        generator.add_dependency(&feat1, &epic1);
        generator.add_dependency(&feat2, &epic1);
        generator.add_dependency(&task1, &feat1);
        generator.add_dependency(&task2, &feat1);
        generator.add_dependency(&task3, &feat2);
        generator.add_dependency(&bug1, &feat1);

        // Set various statuses
        generator.set_status(&epic1, IssueStatus::InProgress);
        generator.set_status(&feat1, IssueStatus::InProgress);
        generator.set_status(&task1, IssueStatus::Closed);
        generator.set_status(&task2, IssueStatus::InProgress);
        generator.set_status(&bug1, IssueStatus::Open);

        // Set priorities
        generator.set_priority(&epic1, Priority::P0);
        generator.set_priority(&bug1, Priority::P0);
        generator.set_priority(&bug2, Priority::P1);

        // Add some more diverse issues
        let chore1 = generator.chore("Update documentation", &["docs"]);
        let chore2 = generator.chore("Refactor auth module", &["backend", "refactor"]);

        let feat3 = generator.feature("Password Reset Flow", &["frontend", "backend", "auth"]);
        generator.add_dependency(&bug2, &feat3);

        // Add estimates
        generator.set_estimate(&task1, 120); // 2 hours
        generator.set_estimate(&task2, 240); // 4 hours
        generator.set_estimate(&feat1, 480); // 8 hours

        // Collect all issues
        issues.extend(generator.issues.values().cloned());

        let label_stats = compute_label_stats(&issues);
        let database_stats = compute_database_stats(&issues);

        Self {
            issues,
            label_stats,
            database_stats,
        }
    }

    /// Generate medium dataset (50 issues)
    fn generate_medium() -> Self {
        let mut generator = IssueGenerator::new(2000);
        let mut issues = Vec::new();

        // Create multiple epics for a realistic project
        let auth_epic = generator.epic("User Authentication & Authorization");
        let dashboard_epic = generator.epic("Dashboard & Analytics");
        let api_epic = generator.epic("REST API Development");
        let testing_epic = generator.epic("Testing & Quality Assurance");

        // Auth features
        let login = generator.feature("Login System", &["frontend", "auth"]);
        let oauth = generator.feature("OAuth Integration", &["backend", "auth", "third-party"]);
        let rbac = generator.feature("Role-Based Access Control", &["backend", "auth", "security"]);

        generator.add_dependency(&login, &auth_epic);
        generator.add_dependency(&oauth, &auth_epic);
        generator.add_dependency(&rbac, &auth_epic);
        generator.add_dependency(&rbac, &login); // RBAC depends on login

        // Dashboard features
        let charts = generator.feature("Interactive Charts", &["frontend", "visualization"]);
        let metrics = generator.feature("Real-time Metrics", &["frontend", "backend", "websocket"]);
        let export = generator.feature("Data Export", &["frontend", "backend"]);

        generator.add_dependency(&charts, &dashboard_epic);
        generator.add_dependency(&metrics, &dashboard_epic);
        generator.add_dependency(&export, &dashboard_epic);

        // API features
        let rest_api = generator.feature("RESTful Endpoints", &["backend", "api"]);
        let graphql = generator.feature("GraphQL Support", &["backend", "api", "graphql"]);
        let rate_limit = generator.feature("Rate Limiting", &["backend", "api", "security"]);

        generator.add_dependency(&rest_api, &api_epic);
        generator.add_dependency(&graphql, &api_epic);
        generator.add_dependency(&rate_limit, &api_epic);

        // Add tasks for each feature (3-5 tasks per feature)
        let features = vec![login.clone(), oauth.clone(), rbac.clone(), charts.clone(),
                            metrics.clone(), export.clone(), rest_api.clone(), graphql.clone(),
                            rate_limit.clone()];

        for (i, feature) in features.iter().enumerate() {
            for j in 0..3 {
                let task_id = generator.task(
                    &format!("Implement {} component {}",
                        generator.issues.get(feature).unwrap().title.split_whitespace().next().unwrap(),
                        j + 1
                    ),
                    &["implementation"]
                );
                generator.add_dependency(&task_id, feature);

                // Mix statuses
                match (i + j) % 4 {
                    0 => generator.set_status(&task_id, IssueStatus::Closed),
                    1 => generator.set_status(&task_id, IssueStatus::InProgress),
                    2 => generator.set_status(&task_id, IssueStatus::Open),
                    _ => generator.set_status(&task_id, IssueStatus::Blocked),
                }
            }
        }

        // Add bugs
        for i in 0..8 {
            let bug_id = generator.bug(
                &format!("Bug: {} edge case", match i % 4 {
                    0 => "Login",
                    1 => "Dashboard",
                    2 => "API",
                    _ => "Export",
                }),
                &["bug", "high-priority"]
            );
            generator.set_priority(&bug_id, if i < 3 { Priority::P0 } else { Priority::P1 });
            generator.set_status(&bug_id, if i % 2 == 0 { IssueStatus::Open } else { IssueStatus::InProgress });
        }

        // Add chores
        for i in 0..5 {
            let chore_id = generator.chore(
                &format!("Chore: {}", match i {
                    0 => "Update dependencies",
                    1 => "Refactor database layer",
                    2 => "Improve error handling",
                    3 => "Add logging",
                    _ => "Update documentation",
                }),
                &["maintenance"]
            );
            generator.set_priority(&chore_id, Priority::P3);
        }

        // Set epic statuses
        generator.set_status(&auth_epic, IssueStatus::InProgress);
        generator.set_status(&dashboard_epic, IssueStatus::InProgress);
        generator.set_status(&api_epic, IssueStatus::Open);
        generator.set_status(&testing_epic, IssueStatus::Open);

        issues.extend(generator.issues.values().cloned());

        let label_stats = compute_label_stats(&issues);
        let database_stats = compute_database_stats(&issues);

        Self {
            issues,
            label_stats,
            database_stats,
        }
    }

    /// Generate large dataset (300 issues) for stress testing
    fn generate_large() -> Self {
        let mut generator = IssueGenerator::new(3000);
        let mut issues = Vec::new();

        // Create 10 epics
        let mut epics = Vec::new();
        for i in 0..10 {
            let epic_id = generator.epic(&format!("Epic {}: Major Feature Area", i + 1));
            generator.set_status(&epic_id, if i < 5 { IssueStatus::InProgress } else { IssueStatus::Open });
            epics.push(epic_id);
        }

        // Create 30 features (3 per epic)
        let mut features = Vec::new();
        for (i, epic_id) in epics.iter().enumerate() {
            for j in 0..3 {
                let feat_id = generator.feature(
                    &format!("Feature {}.{}: Implementation", i + 1, j + 1),
                    &["feature", &format!("area-{}", i)]
                );
                generator.add_dependency(&feat_id, epic_id);
                generator.set_status(&feat_id, match (i + j) % 3 {
                    0 => IssueStatus::Closed,
                    1 => IssueStatus::InProgress,
                    _ => IssueStatus::Open,
                });
                features.push(feat_id);
            }
        }

        // Create 150 tasks (5 per feature)
        for (i, feat_id) in features.iter().enumerate() {
            for j in 0..5 {
                let task_id = generator.task(
                    &format!("Task {}.{}: Implementation detail", i + 1, j + 1),
                    &["task", "implementation"]
                );
                generator.add_dependency(&task_id, feat_id);
                generator.set_status(&task_id, match (i + j) % 4 {
                    0 => IssueStatus::Closed,
                    1 => IssueStatus::InProgress,
                    2 => IssueStatus::Open,
                    _ => IssueStatus::Blocked,
                });
                generator.set_priority(&task_id, match (i + j) % 5 {
                    0 => Priority::P0,
                    1 => Priority::P1,
                    2 => Priority::P2,
                    3 => Priority::P3,
                    _ => Priority::P4,
                });
            }
        }

        // Add 80 bugs
        for i in 0..80 {
            let bug_id = generator.bug(
                &format!("Bug #{}: Critical issue in component", i + 1),
                &["bug", &format!("component-{}", i % 10)]
            );
            generator.set_priority(&bug_id, match i % 5 {
                0 => Priority::P0,
                1 => Priority::P1,
                _ => Priority::P2,
            });
            generator.set_status(&bug_id, if i % 3 == 0 { IssueStatus::Closed } else { IssueStatus::Open });
        }

        // Add 30 chores
        for i in 0..30 {
            let chore_id = generator.chore(
                &format!("Chore #{}: Maintenance task", i + 1),
                &["chore", "maintenance"]
            );
            generator.set_priority(&chore_id, Priority::P3);
        }

        issues.extend(generator.issues.values().cloned());

        let label_stats = compute_label_stats(&issues);
        let database_stats = compute_database_stats(&issues);

        Self {
            issues,
            label_stats,
            database_stats,
        }
    }

    /// Generate deps dataset (60 issues) with complex dependency graphs
    fn generate_deps() -> Self {
        let mut generator = IssueGenerator::new(4000);
        let mut issues = Vec::new();

        // Create a complex dependency tree
        let root_epic = generator.epic("Complete System Redesign");

        // Layer 1: Major components
        let frontend = generator.feature("Frontend Redesign", &["frontend"]);
        let backend = generator.feature("Backend Refactor", &["backend"]);
        let database = generator.feature("Database Migration", &["database"]);
        let api = generator.feature("API Versioning", &["api"]);

        generator.add_dependency(&frontend, &root_epic);
        generator.add_dependency(&backend, &root_epic);
        generator.add_dependency(&database, &root_epic);
        generator.add_dependency(&api, &root_epic);

        // Frontend depends on API
        generator.add_dependency(&frontend, &api);

        // API depends on Backend
        generator.add_dependency(&api, &backend);

        // Backend depends on Database
        generator.add_dependency(&backend, &database);

        // Layer 2: Sub-features
        let ui_components = generator.feature("UI Component Library", &["frontend", "components"]);
        let state_mgmt = generator.feature("State Management", &["frontend", "state"]);
        let routing = generator.feature("Routing System", &["frontend", "routing"]);

        generator.add_dependency(&ui_components, &frontend);
        generator.add_dependency(&state_mgmt, &frontend);
        generator.add_dependency(&routing, &frontend);

        // State management depends on routing
        generator.add_dependency(&state_mgmt, &routing);

        let auth_service = generator.feature("Authentication Service", &["backend", "auth"]);
        let data_layer = generator.feature("Data Access Layer", &["backend", "database"]);
        let cache_layer = generator.feature("Caching Layer", &["backend", "cache"]);

        generator.add_dependency(&auth_service, &backend);
        generator.add_dependency(&data_layer, &backend);
        generator.add_dependency(&cache_layer, &backend);

        // Auth depends on data layer
        generator.add_dependency(&auth_service, &data_layer);

        // Data layer depends on database migration
        generator.add_dependency(&data_layer, &database);

        // Layer 3: Tasks (create chains)
        let components = vec![
            ui_components.clone(), state_mgmt.clone(), routing.clone(),
            auth_service.clone(), data_layer.clone(), cache_layer.clone()
        ];

        for (i, component) in components.iter().enumerate() {
            let mut prev_task: Option<String> = None;
            for j in 0..5 {
                let task_id = generator.task(
                    &format!("Implement step {} for component {}", j + 1, i + 1),
                    &["implementation"]
                );
                generator.add_dependency(&task_id, component);

                if let Some(ref prev) = prev_task {
                    generator.add_dependency(&task_id, prev);
                }
                prev_task = Some(task_id.clone());

                generator.set_status(&task_id, match j {
                    0..=2 => IssueStatus::Closed,
                    3 => IssueStatus::InProgress,
                    _ => IssueStatus::Open,
                });
            }
        }

        // Add some blocked issues (dependencies not ready)
        for i in 0..10 {
            let blocked_task = generator.task(
                &format!("Blocked task waiting on dependencies {}", i + 1),
                &["blocked", "waiting"]
            );
            generator.set_status(&blocked_task, IssueStatus::Blocked);
        }

        issues.extend(generator.issues.values().cloned());

        let label_stats = compute_label_stats(&issues);
        let database_stats = compute_database_stats(&issues);

        Self {
            issues,
            label_stats,
            database_stats,
        }
    }

    /// Generate edge dataset (25 issues) with edge cases
    fn generate_edge() -> Self {
        let mut generator = IssueGenerator::new(5000);
        let mut issues = Vec::new();

        // Unicode and special characters
        let unicode1 = generator.feature("支持中文 Chinese Support", &["i18n", "unicode"]);
        let unicode2 = generator.feature("🚀 Emoji Support 🎉", &["i18n", "emoji"]);
        let unicode3 = generator.task("Тестирование кириллицы", &["testing", "i18n"]);
        let unicode4 = generator.bug("العربية Arabic RTL layout issue", &["bug", "rtl", "i18n"]);

        // Very long title
        let long_title = generator.task(
            "This is an extremely long title that tests how the UI handles issues with very long titles that might wrap multiple lines or get truncated in various views and we want to see how it behaves",
            &["edge-case", "ui-test"]
        );

        // Special characters in titles
        let special1 = generator.bug("Error: `NULL` pointer exception in <Component/>", &["bug", "critical"]);
        let special2 = generator.task("Implement \"quoted\" strings & ampersands", &["feature"]);
        let special3 = generator.chore("Fix issues with \\ backslashes / slashes", &["chore"]);

        // Empty-ish descriptions
        let minimal = generator.task("Minimal task", &[]);

        // Issue with many labels
        let many_labels = generator.feature(
            "Feature with many labels",
            &[
                "label1", "label2", "label3", "label4", "label5",
                "label6", "label7", "label8", "label9", "label10",
                "urgent", "critical", "frontend", "backend", "database"
            ]
        );

        // Very long label names
        let long_labels = generator.task(
            "Task with very long label names",
            &[
                "this-is-a-very-long-label-name-that-tests-ui-wrapping",
                "another-extremely-long-label-for-testing-purposes",
            ]
        );

        // Issue with long description
        let long_desc = generator.epic("Epic with detailed description");
        generator.set_description(&long_desc,
            "This is a very detailed description that spans multiple paragraphs and contains a lot of text.\n\n\
            The purpose of this description is to test how the UI handles long-form content, including:\n\
            - Bullet points\n\
            - Multiple paragraphs\n\
            - Special characters: !@#$%^&*()_+-=[]{}|;':\",./<>?\n\
            - Unicode: 你好 мир العالم 🌍\n\n\
            We want to ensure that all views can properly display and handle this content without breaking the layout."
        );

        // Issues with various priority and status combinations
        let p0_closed = generator.bug("P0 Critical bug (closed)", &["critical"]);
        generator.set_priority(&p0_closed, Priority::P0);
        generator.set_status(&p0_closed, IssueStatus::Closed);

        let p4_in_progress = generator.chore("P4 Low priority chore (in progress)", &["low-priority"]);
        generator.set_priority(&p4_in_progress, Priority::P4);
        generator.set_status(&p4_in_progress, IssueStatus::InProgress);

        // Issue with dependencies to test graph rendering
        let complex_deps = generator.feature("Feature with multiple dependencies", &["complex"]);
        generator.add_dependency(&complex_deps, &unicode1);
        generator.add_dependency(&complex_deps, &unicode2);
        generator.add_dependency(&complex_deps, &long_title);

        // Duplicate titles (edge case for sorting/filtering)
        let dup1 = generator.task("Duplicate Title", &["test"]);
        let dup2 = generator.task("Duplicate Title", &["test"]);
        let dup3 = generator.task("Duplicate Title", &["test"]);

        // Issue with extreme estimates
        let short_estimate = generator.task("Very quick task", &["quick"]);
        generator.set_estimate(&short_estimate, 5); // 5 minutes

        let long_estimate = generator.epic("Multi-month epic");
        generator.set_estimate(&long_estimate, 28800); // 480 hours = 20 days

        issues.extend(generator.issues.values().cloned());

        let label_stats = compute_label_stats(&issues);
        let database_stats = compute_database_stats(&issues);

        Self {
            issues,
            label_stats,
            database_stats,
        }
    }
}

/// Helper struct for generating issues with consistent IDs and relationships
struct IssueGenerator {
    issues: HashMap<String, Issue>,
    next_id: usize,
    base_timestamp: DateTime<Utc>,
    seed: u64,
}

impl IssueGenerator {
    fn new(seed: u64) -> Self {
        Self {
            issues: HashMap::new(),
            next_id: 1,
            base_timestamp: Utc::now() - Duration::days(30), // Start 30 days ago
            seed,
        }
    }

    fn next_issue_id(&mut self) -> String {
        let id = format!("beads-demo-{:04}", self.next_id);
        self.next_id += 1;
        id
    }

    fn create_issue(
        &mut self,
        title: &str,
        issue_type: IssueType,
        labels: &[&str],
    ) -> String {
        let id = self.next_issue_id();

        // Stagger creation times
        let created = self.base_timestamp + Duration::days((self.next_id as i64 - 1) / 5);
        let updated = created + Duration::hours((self.next_id as i64) % 24);

        let issue = Issue {
            id: id.clone(),
            title: title.to_string(),
            status: IssueStatus::Open,
            priority: Priority::P2,
            issue_type,
            description: None,
            assignee: None,
            labels: labels.iter().map(|s| s.to_string()).collect(),
            dependencies: Vec::new(),
            blocks: Vec::new(),
            created,
            updated,
            closed: None,
            notes: Vec::new(),
            est_minutes: None,
            due_date: None,
            defer_date: None,
            close_reason: None,
            external_reference: None,
            flags: IssueFlags::default(),
            design_notes: None,
            acceptance_criteria: None,
            parent_id: None,
            children_ids: Vec::new(),
            event_ids: Vec::new(),
            discovered_ids: Vec::new(),
        };

        self.issues.insert(id.clone(), issue);
        id
    }

    fn epic(&mut self, title: &str) -> String {
        self.create_issue(title, IssueType::Epic, &["epic"])
    }

    fn feature(&mut self, title: &str, labels: &[&str]) -> String {
        self.create_issue(title, IssueType::Feature, labels)
    }

    fn task(&mut self, title: &str, labels: &[&str]) -> String {
        self.create_issue(title, IssueType::Task, labels)
    }

    fn bug(&mut self, title: &str, labels: &[&str]) -> String {
        self.create_issue(title, IssueType::Bug, labels)
    }

    fn chore(&mut self, title: &str, labels: &[&str]) -> String {
        self.create_issue(title, IssueType::Chore, labels)
    }

    fn add_dependency(&mut self, issue_id: &str, depends_on_id: &str) {
        if let Some(issue) = self.issues.get_mut(issue_id) {
            if !issue.dependencies.contains(&depends_on_id.to_string()) {
                issue.dependencies.push(depends_on_id.to_string());
            }
        }
        if let Some(blocked_issue) = self.issues.get_mut(depends_on_id) {
            if !blocked_issue.blocks.contains(&issue_id.to_string()) {
                blocked_issue.blocks.push(issue_id.to_string());
            }
        }
    }

    fn set_status(&mut self, issue_id: &str, status: IssueStatus) {
        if let Some(issue) = self.issues.get_mut(issue_id) {
            issue.status = status;
            if status == IssueStatus::Closed {
                issue.closed = Some(issue.updated + Duration::hours(1));
            }
        }
    }

    fn set_priority(&mut self, issue_id: &str, priority: Priority) {
        if let Some(issue) = self.issues.get_mut(issue_id) {
            issue.priority = priority;
        }
    }

    fn set_estimate(&mut self, issue_id: &str, minutes: u32) {
        if let Some(issue) = self.issues.get_mut(issue_id) {
            issue.est_minutes = Some(minutes);
        }
    }

    fn set_description(&mut self, issue_id: &str, description: &str) {
        if let Some(issue) = self.issues.get_mut(issue_id) {
            issue.description = Some(description.to_string());
        }
    }
}

/// Compute label statistics from a list of issues
fn compute_label_stats(issues: &[Issue]) -> Vec<LabelStats> {
    let mut label_counts: HashMap<String, usize> = HashMap::new();

    for issue in issues {
        for label in &issue.labels {
            *label_counts.entry(label.clone()).or_insert(0) += 1;
        }
    }

    let mut stats: Vec<LabelStats> = label_counts
        .into_iter()
        .map(|(name, count)| LabelStats {
            name,
            count,
            color: None,
            aliases: Vec::new(),
            dimension: None,
            value: None,
        })
        .collect();

    stats.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    stats
}

/// Compute database statistics from a list of issues
fn compute_database_stats(issues: &[Issue]) -> DatabaseStats {
    DatabaseStats {
        total_issues: issues.len(),
        open_issues: issues
            .iter()
            .filter(|i| i.status == IssueStatus::Open)
            .count(),
        in_progress_issues: issues
            .iter()
            .filter(|i| i.status == IssueStatus::InProgress)
            .count(),
        closed_issues: issues
            .iter()
            .filter(|i| i.status == IssueStatus::Closed)
            .count(),
        blocked_issues: issues
            .iter()
            .filter(|i| i.status == IssueStatus::Blocked)
            .count(),
        database_size: 0,
        last_sync: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_small_dataset() {
        let dataset = DemoDataset::generate("small").unwrap();
        assert!(dataset.issues.len() >= 10);
        assert!(dataset.issues.len() <= 20);
        assert!(!dataset.label_stats.is_empty());
        assert!(dataset.database_stats.total_issues > 0);
    }

    #[test]
    fn test_generate_medium_dataset() {
        let dataset = DemoDataset::generate("medium").unwrap();
        assert!(dataset.issues.len() >= 40);
        assert!(dataset.issues.len() <= 60);
    }

    #[test]
    fn test_generate_large_dataset() {
        let dataset = DemoDataset::generate("large").unwrap();
        assert!(dataset.issues.len() >= 250);
        assert!(dataset.issues.len() <= 350);
    }

    #[test]
    fn test_generate_deps_dataset() {
        let dataset = DemoDataset::generate("deps").unwrap();
        assert!(dataset.issues.len() >= 50);
        assert!(dataset.issues.len() <= 70);

        // Should have issues with dependencies
        let has_dependencies = dataset.issues.iter().any(|i| !i.dependencies.is_empty());
        assert!(has_dependencies);
    }

    #[test]
    fn test_generate_edge_dataset() {
        let dataset = DemoDataset::generate("edge").unwrap();
        assert!(dataset.issues.len() >= 20);
        assert!(dataset.issues.len() <= 30);

        // Should have issues with unicode
        let has_unicode = dataset.issues.iter().any(|i| !i.title.is_ascii());
        assert!(has_unicode);
    }

    #[test]
    fn test_deterministic_generation() {
        let dataset1 = DemoDataset::generate("small").unwrap();
        let dataset2 = DemoDataset::generate("small").unwrap();

        assert_eq!(dataset1.issues.len(), dataset2.issues.len());
        // First issue should be the same
        assert_eq!(dataset1.issues[0].title, dataset2.issues[0].title);
        assert_eq!(dataset1.issues[0].issue_type, dataset2.issues[0].issue_type);
    }

    #[test]
    fn test_invalid_dataset_type() {
        let result = DemoDataset::generate("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_database_stats_accuracy() {
        let dataset = DemoDataset::generate("small").unwrap();

        let manual_total = dataset.issues.len();
        let manual_open = dataset.issues.iter().filter(|i| i.status == IssueStatus::Open).count();
        let manual_closed = dataset.issues.iter().filter(|i| i.status == IssueStatus::Closed).count();

        assert_eq!(dataset.database_stats.total_issues, manual_total);
        assert_eq!(dataset.database_stats.open_issues, manual_open);
        assert_eq!(dataset.database_stats.closed_issues, manual_closed);
    }

    #[test]
    fn test_label_stats_computed() {
        let dataset = DemoDataset::generate("small").unwrap();

        // All labeled issues should be counted
        let total_label_instances: usize = dataset.issues.iter()
            .map(|i| i.labels.len())
            .sum();

        let stats_total: usize = dataset.label_stats.iter()
            .map(|s| s.count)
            .sum();

        assert_eq!(total_label_instances, stats_total);
    }
}
