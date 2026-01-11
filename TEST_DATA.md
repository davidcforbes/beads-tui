# Test Data and Fixtures

## Goals
- Provide deterministic datasets for unit and integration tests.
- Cover common workflows and edge cases in a repeatable way.
- Support performance testing with large fixtures.

## Fixture Catalog

| Fixture | Size | Focus |
| --- | --- | --- |
| `beads-mini` | 10-20 issues | Smoke tests and fast unit runs |
| `beads-small` | 200 issues | Daily integration tests |
| `beads-large` | 5,000+ issues | Performance and stress testing |
| `beads-deps` | 60 issues | Complex dependency graphs and cycles |
| `beads-invalid` | 25 issues | Invalid fields and error handling |

## Fixture Contents
Each fixture should cover:
- Status variety (open, in_progress, blocked, closed, deferred).
- Priorities and labels with realistic distributions.
- Long titles and bodies for wrapping and truncation behavior.
- Dependencies with chains, fan-in, fan-out, and cycles.
- Multi-line descriptions and notes.
- Non-ASCII text in a small subset to validate rendering.

## Formats and Storage
- Store fixtures under `tests/fixtures/`.
- Prefer a fixture directory containing a `.beads` database plus a JSONL
  export for diffing and regeneration.
- Include a `fixtures.json` manifest with counts and short descriptions.

## Generation Workflow
1. Create issues in a temporary beads workspace.
2. Export JSONL with `bd export` and copy the `.beads` database.
3. Verify counts and update the manifest.
4. Commit fixtures with a clear version note in the manifest.

## Usage Notes
- Tests should copy fixtures into temp dirs before mutation.
- Integration tests should pin `BD_DB` to the temp fixture path.
- Performance tests should be isolated from unit and snapshot suites.
