use beads_tui::beads::BeadsClient;
use std::process::Command;
use tempfile::TempDir;

pub struct TestHarness {
    pub root: TempDir,
    pub client: BeadsClient,
}

impl TestHarness {
    pub fn new() -> Self {
        let root = TempDir::new().expect("Failed to create temp dir");
        let client = BeadsClient::new().with_cwd(root.path().to_path_buf());
        
        Self { root, client }
    }
    
    pub async fn init(&self) {
        // Initialize a new repository
        let output = Command::new("bd")
            .arg("init")
            .current_dir(self.root.path())
            .output()
            .expect("Failed to run bd init");
            
        assert!(output.status.success(), "bd init failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}
