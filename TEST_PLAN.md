# Test Plan

## Objectives
- Verify critical workflows across issue management, dependencies, and views.
- Protect against regressions in rendering and beads CLI integration.
- Provide clear entry and exit criteria for each release stage.

## Scope
This plan covers all planned UI views and workflows, including the table grid,
kanban board, gantt view, and dependency graph. It includes unit,
integration, snapshot, property-based, and manual testing.

## Coverage Map
- Issue list and table grid: unit, snapshot, integration, manual.
- Issue create/edit forms: unit, integration, snapshot, manual.
- Filters and search: unit, property-based, integration.
- Dependencies (tree, graph, pert): unit, snapshot, integration, manual.
- Kanban and gantt views: snapshot, integration, manual.
- Sync and import/export flows: integration.
- Multi-instance behavior: integration, manual.
- Themes and layouts: snapshot, manual.
- Error handling and retries: unit, integration.

## High Priority Test Scenarios

| ID | Priority | Type | Scenario | Expected |
| --- | --- | --- | --- | --- |
| TP-001 | P1 | Integration | Load fixture DB and open issue list | List renders; navigation works |
| TP-002 | P1 | Unit + Snapshot | Resize and reorder columns | Layout math correct; snapshot stable |
| TP-003 | P1 | Integration | Quick filter by label and status | Results match fixture data |
| TP-004 | P1 | Integration | Create issue via form | Issue appears in list and detail |
| TP-005 | P1 | Integration | Edit issue title and status | List and detail update |
| TP-006 | P1 | Snapshot | Issue list at 80x24 and 120x40 | No clipped headers or rows |
| TP-007 | P1 | Integration | Dependency edit updates graph view | Links and counts update |
| TP-008 | P1 | Integration | Kanban move updates status | Column changes persisted |
| TP-009 | P2 | Snapshot | Gantt view renders lanes and dates | Dates align with fixture |
| TP-010 | P2 | Integration | CLI failure surfaces error state | Error shown; recoverable |
| TP-011 | P2 | Integration | Multi-instance, separate workspaces | No data bleed between runs |
| TP-012 | P2 | Integration | Import/export round-trip | Data preserved after round-trip |

## Entry Criteria
- Feature scope for the milestone is defined in beads.
- Fixtures for the targeted scenarios are available (TEST_DATA.md).
- Test harness runner is available (TEST_HARNESS.md).

## Exit Criteria
- All P1 scenarios pass on Windows, macOS, and Linux.
- Snapshot diffs are reviewed and accepted.
- No open P0 or P1 defects for the milestone.

## Test Data
See TEST_DATA.md for datasets, sizes, and edge-case coverage.

## Reporting
Use beads issues for failures, including environment, reproduction steps,
and expected vs actual results.
