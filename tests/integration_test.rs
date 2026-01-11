//! Integration tests for beads-tui

use beads_tui::beads::BeadsClient;

#[tokio::test]
async fn test_beads_client_creation() {
    let _client = BeadsClient::new();
    assert!(BeadsClient::check_available().is_ok());
}

#[test]
fn test_config_load() {
    // Config should load even if file doesn't exist (returns defaults)
    let result = beads_tui::config::Config::load();
    assert!(result.is_ok());
}
