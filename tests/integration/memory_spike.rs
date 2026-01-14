use crate::common::TestHarness;

#[tokio::test]
async fn test_memory_spike_large_descriptions() {
    let harness = TestHarness::new();
    harness.init().await;

    // Create a large description (10KB of text - more realistic)
    let large_description = "This is a detailed description with lots of context.\n".repeat(200); // ~10KB

    // Create 10 issues with large descriptions
    for i in 0..10 {
        let title = format!("Large Description Issue {}", i);
        let result = harness
            .client
            .create_issue_full(beads_tui::beads::models::CreateIssueParams {
                title: &title,
                issue_type: beads_tui::beads::models::IssueType::Task,
                priority: beads_tui::beads::models::Priority::P2,
                status: None,
                assignee: None,
                labels: &[],
                description: Some(&large_description),
            })
            .await;

        if let Err(e) = result {
            eprintln!("Failed to create issue {}: {:?}", i, e);
            panic!("Failed to create issue with large description: {:?}", e);
        }
    }

    // Now list all issues - this will load all descriptions into memory
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    assert!(
        issues.len() >= 10,
        "Expected at least 10 issues, found {}",
        issues.len()
    );

    // Calculate total memory used by descriptions
    let mut total_desc_size = 0;
    for issue in &issues {
        if let Some(ref desc) = issue.description {
            total_desc_size += desc.len();
        }
    }

    println!(
        "Total description size in memory: {} bytes ({} MB)",
        total_desc_size,
        total_desc_size / (1024 * 1024)
    );

    // With 10 issues at ~10KB each, we should have ~100KB in memory
    assert!(
        total_desc_size >= 10 * 10 * 1024,
        "Expected at least 100KB of descriptions, found {} bytes",
        total_desc_size
    );

    // This test demonstrates that large descriptions cause memory spikes
    // In a real application with hundreds of issues, this could be problematic
    println!(
        "⚠ Memory spike detected: {} issues consume {} MB",
        issues.len(),
        total_desc_size / (1024 * 1024)
    );
}

#[tokio::test]
async fn test_description_truncation_suggestion() {
    let harness = TestHarness::new();
    harness.init().await;

    // Create an issue with a moderate description
    let moderate_desc = "Description line.\n".repeat(100); // ~1.7KB

    harness
        .client
        .create_issue_full(beads_tui::beads::models::CreateIssueParams {
            title: "Moderate Description",
            issue_type: beads_tui::beads::models::IssueType::Task,
            priority: beads_tui::beads::models::Priority::P2,
            status: None,
            assignee: None,
            labels: &[],
            description: Some(&moderate_desc),
        })
        .await
        .expect("Failed to create issue");

    // Load the issue
    let issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    let issue = issues
        .iter()
        .find(|i| i.title == "Moderate Description")
        .unwrap();

    if let Some(ref desc) = issue.description {
        println!("Original description size: {} bytes", desc.len());

        // Suggest truncation strategy: keep first 500 chars for list view
        const TRUNCATION_LIMIT: usize = 500;
        if desc.len() > TRUNCATION_LIMIT {
            let truncated = format!(
                "{}... (truncated, {} more chars)",
                &desc[..TRUNCATION_LIMIT],
                desc.len() - TRUNCATION_LIMIT
            );
            println!("Truncated description size: {} bytes", truncated.len());
            println!("Memory saved: {} bytes", desc.len() - truncated.len());
        }
    }
}
