use crate::common::TestHarness;
use std::process::Command;

#[tokio::test]
async fn test_dependency_cycle_detection() {
    let harness = TestHarness::new();
    harness.init().await;

    // Create cycle: A -> B -> C -> A
    let root = harness.root.path();

    let create = |title: &str| {
        let output = Command::new("bd")
            .args([
                "create",
                "--title",
                title,
                "--type",
                "task",
                "--priority",
                "P2",
            ])
            .current_dir(root)
            .output()
            .expect("Failed to create issue");
        let s = String::from_utf8_lossy(&output.stdout);
        // Extract ID
        s.split_whitespace()
            .find(|part| part.starts_with(".tmp") || part.starts_with("beads-"))
            .unwrap()
            .trim_end_matches(':')
            .to_string()
    };

    let id_a = create("Task A");
    let id_b = create("Task B");
    let id_c = create("Task C");

    let add_dep = |from: &str, to: &str| {
        Command::new("bd")
            .args(["dep", "add", from, to])
            .current_dir(root)
            .status()
            .expect("Failed to add dependency");
    };

    add_dep(&id_a, &id_b);
    add_dep(&id_b, &id_c);
    add_dep(&id_c, &id_a); // Cycle!

    // Load into state
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    // Check if app logic detects cycle (assuming we have a cycle detection helper)
    // For now, we verify that we can at least list them without crashing.
    assert_eq!(issues.len(), 3);
}

#[tokio::test]
async fn test_deep_dependency_chain() {
    let harness = TestHarness::new();
    harness.init().await;
    let root = harness.root.path();

    let mut prev_id: Option<String> = None;
    for i in 0..20 {
        let title = format!("Chain Task {}", i);
        let id = Command::new("bd")
            .args([
                "create",
                "--title",
                &title,
                "--type",
                "task",
                "--priority",
                "P2",
            ])
            .current_dir(root)
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .find(|p| p.contains(".tmp") || p.contains("beads-"))
                    .unwrap()
                    .trim_end_matches(':')
                    .to_string()
            })
            .expect("Failed to create issue");

        if let Some(p) = prev_id {
            Command::new("bd")
                .args(["dep", "add", &p, &id])
                .current_dir(root)
                .status()
                .unwrap();
        }
        prev_id = Some(id);
    }

    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list chain");
    assert_eq!(issues.len(), 20);
}
