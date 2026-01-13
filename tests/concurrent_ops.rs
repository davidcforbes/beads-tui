mod common;
use common::TestHarness;
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
}
