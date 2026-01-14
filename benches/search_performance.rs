use beads_tui::beads::models::{Issue, IssueStatus, IssueType, Note, Priority};
use beads_tui::models::IssueFilter;
use beads_tui::ui::views::{SearchInterfaceState, SearchScope};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Create a test issue for benchmarking with varied content
fn create_test_issue(id: u32) -> Issue {
    Issue {
        id: format!("beads-test-{id:05}"),
        title: format!("Issue {id}: {}", get_varied_title(id)),
        description: Some(format!(
            "Detailed description for issue {id}. This issue relates to {}. \
             Additional context: {}.",
            get_varied_context(id),
            get_varied_details(id)
        )),
        issue_type: match id % 5 {
            0 => IssueType::Bug,
            1 => IssueType::Feature,
            2 => IssueType::Task,
            3 => IssueType::Epic,
            _ => IssueType::Chore,
        },
        status: match id % 4 {
            0 => IssueStatus::Open,
            1 => IssueStatus::InProgress,
            2 => IssueStatus::Blocked,
            _ => IssueStatus::Closed,
        },
        priority: match id % 5 {
            0 => Priority::P0,
            1 => Priority::P1,
            2 => Priority::P2,
            3 => Priority::P3,
            _ => Priority::P4,
        },
        labels: vec![format!("component-{}", id % 8), format!("area-{}", id % 6)],
        assignee: if id % 3 == 0 {
            Some(format!("developer{}", id % 5))
        } else {
            None
        },
        created: Utc::now(),
        updated: Utc::now(),
        closed: None,
        dependencies: vec![],
        blocks: vec![],
        notes: vec![
            Note {
                timestamp: Utc::now(),
                author: "benchmark".to_string(),
                content: format!("Note 1 for issue {id}"),
            },
            Note {
                timestamp: Utc::now(),
                author: "benchmark".to_string(),
                content: format!("Follow-up required: {}", id % 2 == 0),
            },
        ],
    }
}

fn get_varied_title(id: u32) -> &'static str {
    match id % 10 {
        0 => "Fix authentication bug in login flow",
        1 => "Implement user profile management",
        2 => "Add search functionality to dashboard",
        3 => "Refactor database connection pooling",
        4 => "Update documentation for API endpoints",
        5 => "Optimize query performance",
        6 => "Fix memory leak in background worker",
        7 => "Add unit tests for validation logic",
        8 => "Improve error handling in parser",
        _ => "General maintenance task",
    }
}

fn get_varied_context(id: u32) -> &'static str {
    match id % 8 {
        0 => "authentication and security",
        1 => "user interface components",
        2 => "database optimization",
        3 => "API endpoints",
        4 => "testing infrastructure",
        5 => "performance improvements",
        6 => "documentation updates",
        _ => "general maintenance",
    }
}

fn get_varied_details(id: u32) -> &'static str {
    match id % 6 {
        0 => "requires coordination with backend team",
        1 => "breaking change, needs migration path",
        2 => "low priority, can be deferred",
        3 => "critical fix, needs immediate attention",
        4 => "enhancement requested by multiple users",
        _ => "routine maintenance work",
    }
}

/// Create a batch of test issues
fn create_issues_batch(count: usize) -> Vec<Issue> {
    (0..count).map(|i| create_test_issue(i as u32)).collect()
}

/// Benchmark substring search across all fields (realistic scenario)
fn bench_search_all_fields_substring(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_all_fields_substring");

    for size in [100, 500, 1000, 5000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                state.search_state_mut().set_query("authentication");
                state.update_filtered_issues();
                black_box(state.filtered_issues().len())
            });
        });
    }

    group.finish();
}

/// Benchmark fuzzy search performance
fn bench_search_fuzzy_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_fuzzy_matching");

    for size in [100, 500, 1000, 5000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                if !state.is_fuzzy_enabled() {
                    state.toggle_fuzzy();
                }
                state.search_state_mut().set_query("athntctn"); // typo: "authentication"
                state.update_filtered_issues();
                black_box(state.filtered_issues().len())
            });
        });
    }

    group.finish();
}

/// Benchmark regex search performance
fn bench_search_regex_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_regex_matching");

    for size in [100, 500, 1000, 5000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                if !state.is_regex_enabled() {
                    state.toggle_regex();
                }
                state.search_state_mut().set_query("(auth|login|security)");
                state.update_filtered_issues();
                black_box(state.filtered_issues().len())
            });
        });
    }

    group.finish();
}

/// Benchmark repeated search with query caching (tests optimization)
fn bench_search_with_query_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_with_query_cache");

    for size in [100, 500, 1000, 5000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                state.search_state_mut().set_query("performance");

                // First search (populates cache)
                state.update_filtered_issues();
                let result1 = state.filtered_issues().len();

                // Simulate issue list update without query change
                state.set_issues(issues.clone());

                // Second search (uses cache)
                let result2 = state.filtered_issues().len();

                black_box((result1, result2))
            });
        });
    }

    group.finish();
}

/// Benchmark search with filter combination
fn bench_search_with_status_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_with_status_filter");

    for size in [100, 500, 1000, 5000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                state.search_state_mut().set_query("bug");
                let mut filter = IssueFilter::new();
                filter.status = Some(IssueStatus::Open);
                state.apply_filter(&filter);
                state.update_filtered_issues();
                black_box(state.filtered_issues().len())
            });
        });
    }

    group.finish();
}

/// Benchmark case-insensitive search (most common use case)
fn bench_search_case_insensitive(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_case_insensitive");

    for size in [100, 500, 1000, 5000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                state.search_state_mut().set_query("AUTHENTICATION"); // uppercase query
                state.update_filtered_issues();
                black_box(state.filtered_issues().len())
            });
        });
    }

    group.finish();
}

/// Benchmark search on large datasets (stress test)
fn bench_search_large_datasets(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_large_datasets");
    group.sample_size(20); // Reduce sample size for very large datasets

    for size in [5000, 10000].iter() {
        let issues = create_issues_batch(*size);

        // Substring search
        group.bench_with_input(BenchmarkId::new("substring", size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                state.search_state_mut().set_query("performance");
                state.update_filtered_issues();
                black_box(state.filtered_issues().len())
            });
        });

        // Fuzzy search
        group.bench_with_input(BenchmarkId::new("fuzzy", size), size, |b, _| {
            b.iter(|| {
                let mut state = SearchInterfaceState::new(issues.clone());
                if !state.is_fuzzy_enabled() {
                    state.toggle_fuzzy();
                }
                state.search_state_mut().set_query("prfmnc"); // typo
                state.update_filtered_issues();
                black_box(state.filtered_issues().len())
            });
        });
    }

    group.finish();
}

/// Benchmark search scope variations
fn bench_search_different_scopes(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_different_scopes");
    let issues = create_issues_batch(1000);

    group.bench_function("title_only", |b| {
        b.iter(|| {
            let mut state = SearchInterfaceState::new(issues.clone());
            state.set_search_scope(SearchScope::Title);
            state.search_state_mut().set_query("authentication");
            state.update_filtered_issues();
            black_box(state.filtered_issues().len())
        });
    });

    group.bench_function("description_only", |b| {
        b.iter(|| {
            let mut state = SearchInterfaceState::new(issues.clone());
            state.set_search_scope(SearchScope::Description);
            state.search_state_mut().set_query("authentication");
            state.update_filtered_issues();
            black_box(state.filtered_issues().len())
        });
    });

    group.bench_function("all_fields", |b| {
        b.iter(|| {
            let mut state = SearchInterfaceState::new(issues.clone());
            state.set_search_scope(SearchScope::All);
            state.search_state_mut().set_query("authentication");
            state.update_filtered_issues();
            black_box(state.filtered_issues().len())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_search_all_fields_substring,
    bench_search_fuzzy_matching,
    bench_search_regex_matching,
    bench_search_with_query_cache,
    bench_search_with_status_filter,
    bench_search_case_insensitive,
    bench_search_large_datasets,
    bench_search_different_scopes,
);

criterion_main!(benches);
