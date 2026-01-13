use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct FixtureManifest {
    fixtures: Vec<FixtureEntry>,
}

#[derive(Deserialize)]
struct FixtureEntry {
    name: String,
    path: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn fixtures_manifest_lists_existing_fixtures() {
    let manifest_path = repo_root().join("tests/fixtures/fixtures.json");
    let manifest_text = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|_| panic!("Missing fixture manifest: {}", manifest_path.display()));
    let manifest: FixtureManifest = serde_json::from_str(&manifest_text)
        .unwrap_or_else(|err| panic!("Invalid fixture manifest JSON: {err}"));

    assert!(
        !manifest.fixtures.is_empty(),
        "fixtures.json must list at least one fixture"
    );

    for fixture in manifest.fixtures {
        let fixture_path = repo_root().join(&fixture.path);
        assert!(
            fixture_path.is_dir(),
            "Fixture directory missing for {} at {}",
            fixture.name,
            fixture_path.display()
        );

        let fixture_json_path = fixture_path.join("fixture.json");
        let issues_jsonl_path = fixture_path.join("issues.jsonl");
        let readme_path = fixture_path.join("README.md");
        let beads_dir = fixture_path.join(".beads");

        assert!(
            fixture_json_path.is_file(),
            "fixture.json missing for {} at {}",
            fixture.name,
            fixture_json_path.display()
        );
        assert!(
            issues_jsonl_path.is_file(),
            "issues.jsonl missing for {} at {}",
            fixture.name,
            issues_jsonl_path.display()
        );
        assert!(
            readme_path.is_file(),
            "README.md missing for {} at {}",
            fixture.name,
            readme_path.display()
        );
        assert!(
            beads_dir.is_dir(),
            ".beads directory missing for {} at {}",
            fixture.name,
            beads_dir.display()
        );

        let fixture_json_text = fs::read_to_string(&fixture_json_path)
            .unwrap_or_else(|_| panic!("Unable to read {}", fixture_json_path.display()));
        let fixture_value: serde_json::Value = serde_json::from_str(&fixture_json_text)
            .unwrap_or_else(|err| panic!("Invalid fixture.json for {}: {err}", fixture.name));
        let name_value = fixture_value
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        assert_eq!(
            name_value, fixture.name,
            "fixture.json name mismatch for {}",
            fixture.name
        );

        let issues_text = fs::read_to_string(&issues_jsonl_path)
            .unwrap_or_else(|_| panic!("Unable to read {}", issues_jsonl_path.display()));
        assert!(
            issues_text.lines().any(|line| !line.trim().is_empty()),
            "issues.jsonl is empty for {}",
            fixture.name
        );
    }
}
