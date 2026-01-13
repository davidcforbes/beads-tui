# Release Process

This document defines the manual release flow for beads-tui.

## Preconditions
- Working tree clean on main
- All tests passing
- Release version agreed
- Access to GitHub releases and crates.io
- Packaging tools installed (if distributing binaries)

## Steps
1. Sync repo:
   - git pull --rebase
   - git status
2. Update version:
   - Cargo.toml version
   - Any docs that mention the version
3. Update release notes:
   - CHANGELOG.md if applicable
   - Capture notable changes for GitHub release notes
4. Run quality gates:
   - scripts/validate-fixtures.ps1
   - scripts/test.ps1 -Suite all
   - cargo fmt --check
   - cargo clippy -- -D warnings
5. Build release artifacts:
   - cargo build --release
   - verify target/release/beads-tui(.exe) runs
6. Tag release:
   - git tag -a vX.Y.Z -m "release vX.Y.Z"
   - git push origin main --tags
7. Publish:
   - Create GitHub release with notes
   - Upload artifacts if available
   - cargo publish (if crates.io is enabled)
8. Post-release verification:
   - Install from release artifact
   - Smoke test basic flows
   - Update any tracking issues

## Backout
If a release is bad:
- Yank the crates.io version (cargo yank)
- Mark GitHub release as pre-release or retract
- File a beads bug with impact and fix plan
