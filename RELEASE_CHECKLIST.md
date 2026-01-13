# Release Checklist

For details, see RELEASE_PROCESS.md.

## Pre-release
- [ ] main branch up to date and clean
- [ ] Version bump applied in Cargo.toml
- [ ] CHANGELOG.md updated (if applicable)
- [ ] Release notes drafted
- [ ] scripts/validate-fixtures.ps1 passes
- [ ] scripts/test.ps1 -Suite all passes
- [ ] cargo fmt --check
- [ ] cargo clippy -- -D warnings

## Build
- [ ] cargo build --release
- [ ] Binary runs locally
- [ ] Artifacts staged for distribution

## Publish
- [ ] Git tag created (vX.Y.Z)
- [ ] git push origin main --tags
- [ ] GitHub release created with notes
- [ ] Artifacts uploaded
- [ ] cargo publish (if applicable)

## Post-release
- [ ] Install from published artifact
- [ ] Smoke test core flows
- [ ] Update beads issues/epic status
