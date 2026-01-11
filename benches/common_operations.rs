use beads_tui::beads::models::{Issue, IssueStatus, IssueType, Priority};
use beads_tui::beads::parser::parse_issue_list;
use beads_tui::ui::widgets::issue_list::{IssueList, IssueListState, SortColumn, SortDirection};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use chrono::Utc;

/// Create a test issue for benchmarking
fn create_test_issue(id: u32) -> Issue {
    Issue {
        id: format!("beads-test-{:04}", id),
        title: format!("Test Issue {}", id),
        description: Some(format!("Description for issue {}", id)),
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
        labels: vec![format!("label-{}", id % 10)],
        assignee: if id % 3 == 0 {
            Some(format!("user{}", id % 5))
        } else {
            None
        },
        created: Utc::now(),
        updated: Utc::now(),
        closed: None,
        dependencies: vec![],
        blocks: vec![],
        notes: vec![],
    }
}

/// Create a batch of test issues
fn create_issues_batch(count: usize) -> Vec<Issue> {
    (0..count).map(|i| create_test_issue(i as u32)).collect()
}

/// Benchmark issue filtering by status
fn bench_filter_by_status(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_by_status");

    for size in [10, 100, 500, 1000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filtered: Vec<_> = issues
                    .iter()
                    .filter(|issue| issue.status == IssueStatus::Open)
                    .collect();
                black_box(filtered)
            });
        });
    }

    group.finish();
}

/// Benchmark issue filtering by multiple criteria
fn bench_filter_multiple_criteria(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_multiple_criteria");

    for size in [10, 100, 500, 1000].iter() {
        let issues = create_issues_batch(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filtered: Vec<_> = issues
                    .iter()
                    .filter(|issue| {
                        issue.status == IssueStatus::Open
                            && issue.priority == Priority::P1
                            && issue.issue_type == IssueType::Bug
                    })
                    .collect();
                black_box(filtered)
            });
        });
    }

    group.finish();
}

/// Benchmark issue search by title
fn bench_search_by_title(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_by_title");

    for size in [10, 100, 500, 1000].iter() {
        let issues = create_issues_batch(*size);
        let query = "Test Issue 42";

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let found: Vec<_> = issues
                    .iter()
                    .filter(|issue| issue.title.contains(query))
                    .collect();
                black_box(found)
            });
        });
    }

    group.finish();
}

/// Benchmark case-insensitive search
fn bench_search_case_insensitive(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_case_insensitive");

    for size in [10, 100, 500, 1000].iter() {
        let issues = create_issues_batch(*size);
        let query = "test issue";
        let query_lower = query.to_lowercase();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let found: Vec<_> = issues
                    .iter()
                    .filter(|issue| issue.title.to_lowercase().contains(&query_lower))
                    .collect();
                black_box(found)
            });
        });
    }

    group.finish();
}

/// Benchmark issue list sorting
fn bench_issue_list_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_list_sorting");

    for size in [10, 100, 500, 1000].iter() {
        let issues = create_issues_batch(*size);
        let issue_refs: Vec<&Issue> = issues.iter().collect();

        // Benchmark sort by priority
        group.bench_with_input(
            BenchmarkId::new("by_priority", size),
            size,
            |b, _| {
                b.iter(|| {
                    let list = IssueList::new(issue_refs.clone())
                        .with_sort(SortColumn::Priority, SortDirection::Ascending);
                    black_box(list)
                });
            },
        );

        // Benchmark sort by title
        group.bench_with_input(
            BenchmarkId::new("by_title", size),
            size,
            |b, _| {
                b.iter(|| {
                    let list = IssueList::new(issue_refs.clone())
                        .with_sort(SortColumn::Title, SortDirection::Ascending);
                    black_box(list)
                });
            },
        );

        // Benchmark sort by updated
        group.bench_with_input(
            BenchmarkId::new("by_updated", size),
            size,
            |b, _| {
                b.iter(|| {
                    let list = IssueList::new(issue_refs.clone())
                        .with_sort(SortColumn::Updated, SortDirection::Descending);
                    black_box(list)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parsing issue list output from JSON
fn bench_parse_list_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_json_list");

    // Generate JSON output from issues
    let sample_outputs = vec![
        (10, generate_json_output(10)),
        (100, generate_json_output(100)),
        (500, generate_json_output(500)),
    ];

    for (size, output) in sample_outputs.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), output, |b, output| {
            b.iter(|| {
                let parsed = parse_issue_list(black_box(output));
                black_box(parsed)
            });
        });
    }

    group.finish();
}

/// Generate sample JSON output for issue list
fn generate_json_output(count: usize) -> String {
    let issues: Vec<_> = (0..count).map(|i| create_test_issue(i as u32)).collect();
    serde_json::to_string(&issues).unwrap()
}

/// Benchmark issue list state navigation
fn bench_issue_list_navigation(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_list_navigation");

    for size in [10, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            b.iter(|| {
                let mut state = IssueListState::new();
                // Navigate through all issues
                for _ in 0..*size {
                    state.select_next(*size);
                }
                black_box(state)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_filter_by_status,
    bench_filter_multiple_criteria,
    bench_search_by_title,
    bench_search_case_insensitive,
    bench_issue_list_sorting,
    bench_parse_list_output,
    bench_issue_list_navigation,
);

criterion_main!(benches);
