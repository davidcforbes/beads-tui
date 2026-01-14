use crate::common::TestHarness;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[tokio::test]
async fn test_concurrent_modifications() {
    let harness = TestHarness::new();
    harness.init().await;

    let root_path = harness.root.path().to_path_buf();
    let root_path_clone = root_path.clone();

    // Thread 1: Create issues continuously via CLI
    let handle = thread::spawn(move || {
        for i in 0..10 {
            let title = format!("Concurrent Task {}", i);
            let status = Command::new("bd")
                .args([
                    "create",
                    "--title",
                    &title,
                    "--type",
                    "task",
                    "--priority",
                    "P2",
                ])
                .current_dir(&root_path_clone)
                .status()
                .expect("Failed to execute bd create");

            assert!(status.success());
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Main Thread: List issues continuously using Client
    for _ in 0..10 {
        let issues = harness.client.list_issues(None, None).await;
        // We expect occasional failures due to lock contention if not handled,
        // but the client should ideally retry or return an error we can catch.
        // For now, we just assert that we can eventually read.

        match issues {
            Ok(list) => {
                println!("Read {} issues", list.len());
            }
            Err(e) => {
                println!("Read failed (expected contention): {:?}", e);
            }
        }
        tokio::time::sleep(Duration::from_millis(60)).await;
    }

    handle.join().unwrap();

    // Final verification
    let final_issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Final read failed");
    assert!(
        final_issues.len() >= 10,
        "Expected at least 10 issues, found {}",
        final_issues.len()
    );

    // Verify all IDs are unique (no duplicates)
    let mut ids: Vec<String> = final_issues.iter().map(|i| i.id.clone()).collect();
    ids.sort();
    let original_len = ids.len();
    ids.dedup();
    assert_eq!(
        ids.len(),
        original_len,
        "Found duplicate IDs! {} unique IDs out of {} total",
        ids.len(),
        original_len
    );

    // Verify all IDs are valid (not empty)
    for issue in &final_issues {
        assert!(
            !issue.id.is_empty(),
            "Found issue with empty ID: {:?}",
            issue.title
        );
        assert!(
            issue.id.len() > 5,
            "Found issue with suspiciously short ID: {}",
            issue.id
        );
    }
}

#[tokio::test]
async fn test_concurrent_create_race_condition() {
    let harness = TestHarness::new();
    harness.init().await;

    let root_path = harness.root.path().to_path_buf();

    // Spawn multiple threads that all create issues simultaneously
    let handles: Vec<_> = (0..5)
        .map(|thread_id| {
            let root = root_path.clone();
            thread::spawn(move || {
                let mut created_ids = Vec::new();
                for i in 0..5 {
                    let title = format!("Thread {} Task {}", thread_id, i);
                    let output = Command::new("bd")
                        .args([
                            "create",
                            "--title",
                            &title,
                            "--type",
                            "task",
                            "--priority",
                            "P2",
                        ])
                        .current_dir(&root)
                        .output()
                        .expect("Failed to execute bd create");

                    assert!(output.status.success(), "Create failed for {}", title);

                    // Extract ID from output (format: "✓ Created issue: <ID>")
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.contains("Created issue:") {
                            // Extract everything after "Created issue: "
                            if let Some(pos) = line.find("Created issue:") {
                                let id_part = &line[pos + 15..]; // Skip "Created issue: "
                                let id = id_part.trim().split_whitespace().next().unwrap_or("");
                                if !id.is_empty() {
                                    created_ids.push(id.to_string());
                                }
                            }
                        }
                    }
                }
                created_ids
            })
        })
        .collect();

    // Collect all created IDs from all threads
    let mut all_ids = Vec::new();
    for handle in handles {
        let ids = handle.join().unwrap();
        all_ids.extend(ids);
    }

    println!("Created {} issues total", all_ids.len());
    assert_eq!(all_ids.len(), 25, "Should have created exactly 25 issues");

    // Critical check: Verify NO duplicate IDs
    let mut sorted_ids = all_ids.clone();
    sorted_ids.sort();
    for i in 0..sorted_ids.len() - 1 {
        if sorted_ids[i] == sorted_ids[i + 1] {
            panic!(
                "RACE CONDITION DETECTED! Duplicate ID found: {}",
                sorted_ids[i]
            );
        }
    }

    // Verify with final list from database
    let final_issues = harness
        .client
        .list_issues(None, None)
        .await
        .expect("Failed to list issues");

    assert!(
        final_issues.len() >= 25,
        "Database shows {} issues, expected at least 25",
        final_issues.len()
    );

    // Verify all database IDs are unique
    let db_ids: Vec<String> = final_issues.iter().map(|i| i.id.clone()).collect();
    let unique_count = db_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(
        unique_count,
        db_ids.len(),
        "Database has duplicate IDs! {} unique out of {} total",
        unique_count,
        db_ids.len()
    );

    println!(
        "✓ Race condition test passed: All {} IDs are unique",
        all_ids.len()
    );
}
