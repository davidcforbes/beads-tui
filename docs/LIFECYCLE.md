# Issue Lifecycle Workflows

This guide explains how issues flow through their lifecycle in beads-tui, from creation to completion.

## Overview

Issues in beads-tui follow a defined lifecycle with status transitions, type-specific workflows, and various operations that can be performed at each stage. Understanding these workflows will help you manage your project efficiently.

## Issue Status Flow

### Basic Status Lifecycle

```
Open → In Progress → Closed
  ↓                     ↑
Blocked ---------------┘
```

### Status Definitions

- **Open**: Issue is created and ready to be worked on (no blockers)
- **In Progress**: Issue is actively being worked on
- **Blocked**: Issue cannot proceed due to dependencies or external factors
- **Closed**: Issue is completed and verified

### Status Transitions

#### Open → In Progress
Transition when you start working on an issue.

**In the UI**:
1. Select the issue in the list view
2. Press `e` to edit
3. Change status to "In Progress"
4. Press `Ctrl+S` to save

**Via CLI**:
```bash
bd update <issue-id> --status in_progress
```

#### In Progress → Closed
Transition when work is complete and verified.

**In the UI**:
1. Select the issue in the list view
2. Press `c` to close
3. Optionally add a closing reason
4. Confirm the action

**Via CLI**:
```bash
bd close <issue-id> --reason "Implemented and tested"
```

#### Open/In Progress → Blocked
Transition when progress is halted by dependencies or external factors.

**In the UI**:
1. Select the issue
2. Press `e` to edit
3. Change status to "Blocked"
4. Add a comment explaining the blocker
5. Add dependency relationships if needed

**Via CLI**:
```bash
bd update <issue-id> --status blocked
bd comment <issue-id> "Blocked by missing API endpoint"
bd dep add <issue-id> <blocking-issue-id>
```

#### Blocked → In Progress
Transition when blocker is resolved.

**In the UI**:
1. Verify blocking issues are resolved
2. Select the issue
3. Press `e` to edit
4. Change status to "In Progress"

#### Closed → Open
Reopen an issue when additional work is needed.

**In the UI**:
1. Select the closed issue
2. Press `r` to reopen
3. Add a comment explaining why
4. Issue status changes to "Open"

**Via CLI**:
```bash
bd reopen <issue-id> --reason "Found regression in testing"
```

## Issue Type Workflows

Different issue types follow similar lifecycles but with type-specific best practices.

### Epic Workflow

Epics are large initiatives broken down into smaller issues.

#### Creation
1. Press `n` to create new issue
2. Set type to "Epic"
3. Write a comprehensive description
4. Set priority (typically P1 or P2)
5. Save with `Ctrl+S`

#### Breakdown
1. Create child issues (features, tasks, bugs)
2. Add dependencies: `bd dep add <child-id> <epic-id>`
3. Track progress through dependency tree

#### Progress Tracking
- View dependency tree: Press `d` on the epic
- Check completion percentage
- Monitor blocked child issues

#### Completion
- All child issues must be closed first
- Verify all goals achieved
- Close epic with summary

### Feature Workflow

Features represent new functionality.

#### Planning Phase (Open)
1. Create feature issue
2. Write user stories or requirements
3. Add design notes or mockups
4. Break down into tasks if large
5. Add dependencies on prerequisite work

#### Development Phase (In Progress)
1. Mark as in progress
2. Create git branch: `feature/<issue-id>-description`
3. Implement functionality
4. Update issue with progress notes
5. Link related commits

#### Review Phase
1. Create pull request
2. Add PR link to issue comments
3. Address review feedback
4. Update tests

#### Completion
1. Merge PR
2. Verify in production/staging
3. Close issue with summary
4. Update related documentation

### Task Workflow

Tasks are concrete, well-defined work items.

#### Execution (Open → In Progress)
1. Review task requirements
2. Mark as in progress
3. Complete the work
4. Run tests

#### Verification
1. Self-review changes
2. Run relevant test suites
3. Check acceptance criteria

#### Completion
1. Commit changes
2. Close task
3. Update parent issue if applicable

### Bug Workflow

Bugs represent defects or issues in existing functionality.

#### Triage Phase (Open)
1. Reproduce the bug
2. Add reproduction steps to description
3. Set priority based on severity:
   - P0: Critical (blocking production)
   - P1: High (significant impact)
   - P2: Medium (moderate impact)
   - P3: Low (minor issue)
   - P4: Backlog (cosmetic)
4. Add labels (e.g., "ui", "backend", "performance")

#### Investigation Phase (In Progress)
1. Mark as in progress
2. Add investigation notes as comments
3. Identify root cause
4. If blocked by missing info, mark as Blocked

#### Fix Phase
1. Implement fix
2. Add regression test
3. Verify fix resolves issue
4. Test for side effects

#### Verification Phase
1. Deploy to test environment
2. Verify fix works
3. Run full test suite
4. Check related functionality

#### Completion
1. Close bug with fix summary
2. Document root cause (if useful)
3. Add follow-up issues if needed

### Chore Workflow

Chores are maintenance or infrastructure work.

#### Execution
1. Similar to task workflow
2. Focus on non-user-facing improvements
3. May include:
   - Refactoring
   - Dependency updates
   - Documentation
   - CI/CD improvements

#### Completion
1. Verify changes don't break functionality
2. Run full test suite
3. Close with summary of changes

## Dependency Management

### Adding Dependencies

When an issue depends on another:

**In the UI**:
1. View the dependent issue
2. Press `d` to open dependency editor
3. Press `a` to add dependency
4. Search for or enter blocking issue ID
5. Select dependency type:
   - **Blocks**: This issue cannot start until blocker completes
   - **Related**: Informational relationship
   - **Discovered From**: Issue was discovered while working on another

**Via CLI**:
```bash
bd dep add <dependent-id> <blocker-id>
```

### Viewing Dependency Trees

**In the UI**:
1. Select issue with dependencies
2. Press `d` to view dependency tree
3. Navigate with arrow keys
4. Press `Enter` to view details
5. Expand/collapse nodes with `Space`

**Via CLI**:
```bash
bd dep tree <issue-id>
```

### Resolving Circular Dependencies

If you create a circular dependency, beads will prevent it:

1. Review the dependency error message
2. Identify which dependency creates the cycle
3. Remove or reorganize dependencies
4. Consider breaking issues into smaller pieces

## Label Management

### Adding Labels

**In the UI**:
1. Select issue
2. Press `l` to open label manager
3. Type to search labels
4. Press `Space` to toggle label
5. Press `Enter` to save

**Via CLI**:
```bash
bd label add <issue-id> frontend,ui,react
```

### Label Strategies

**By Area**:
- `ui`, `backend`, `frontend`, `database`, `api`

**By Technology**:
- `rust`, `react`, `typescript`, `sql`

**By Status**:
- `needs-review`, `needs-testing`, `blocked-external`

**By Priority/Urgency**:
- `critical`, `urgent`, `tech-debt`

**By Effort**:
- `quick-win`, `large-effort`, `complex`

## Bulk Operations

### Closing Multiple Issues

When completing related work:

**In the UI**:
1. In list view, press `Space` to toggle selection
2. Select all issues to close
3. Press `c` to close selected
4. Confirm bulk action

**Via CLI**:
```bash
bd close <id1> <id2> <id3> --reason "Completed in sprint 5"
```

### Bulk Label Updates

**Via CLI**:
```bash
# Add label to multiple issues
bd label add <id1> <id2> <id3> sprint-5

# Remove label from multiple issues
bd label remove <id1> <id2> <id3> blocked
```

### Bulk Status Updates

**Via CLI**:
```bash
# Start multiple issues
bd update <id1> <id2> <id3> --status in_progress

# Close multiple completed issues
bd close <id1> <id2> <id3>
```

## Advanced Workflows

### Sprint Planning

1. Create sprint epic: `Sprint 2026-01`
2. Identify issues for sprint
3. Add dependencies: `bd dep add <issue-id> <sprint-epic-id>`
4. Review dependency tree for blockers
5. Mark selected issues as in_progress
6. Track progress through sprint

### Release Planning

1. Create release epic: `v1.0.0 Release`
2. Add all release-critical features/bugs as dependencies
3. Use PERT chart view to see critical path
4. Identify parallel work opportunities
5. Monitor blocked issues
6. Close release epic when all dependencies complete

### Hotfix Workflow

For critical production issues:

1. Create P0 bug issue
2. Add `hotfix` label
3. Mark as in_progress immediately
4. Create hotfix branch
5. Implement minimal fix
6. Test thoroughly
7. Deploy to production
8. Close issue with deployment notes
9. Create follow-up issues for proper fix if needed

### Feature Flags

When using feature flags:

1. Create feature issue
2. Add `feature-flag` label
3. Implement behind flag
4. Deploy to production (flag off)
5. Mark issue as "In Progress" (not closed)
6. Test with flag on
7. Gradually enable flag
8. Once fully rolled out, close issue
9. Create cleanup issue to remove flag

## Best Practices

### 1. Keep Status Current

Update issue status as work progresses. This helps:
- Team visibility into project status
- Identifying bottlenecks
- Planning future work

### 2. Add Context in Comments

When changing status, add a comment explaining:
- Why moved to Blocked
- What was completed before closing
- Why reopening closed issue

### 3. Use Dependencies Effectively

- Add dependencies to track blockers
- Review dependency trees before planning
- Break circular dependencies by refactoring

### 4. Close Issues Promptly

- Don't let completed work sit in "In Progress"
- Close issues as soon as verification is complete
- Add closing reason for context

### 5. Label Consistently

- Establish team labeling conventions
- Use labels for filtering and reporting
- Avoid label proliferation

### 6. Review Blocked Issues Regularly

- Check blocked issues weekly
- Update status when blockers resolve
- Add comments on blocker status

### 7. Break Down Large Issues

- If an issue stays in progress for weeks, break it down
- Create child issues for sub-tasks
- Track progress through parent issue

### 8. Use Search and Filters

- Find old issues with `/` search
- Filter by status to find stale work
- Use saved filters for common views

## Keyboard Shortcuts

### Issue List View

- `n` - Create new issue
- `e` - Edit selected issue
- `c` - Close selected issue
- `r` - Reopen closed issue
- `d` - View dependencies
- `l` - Manage labels
- `Space` - Toggle selection (bulk mode)
- `Enter` - View issue details
- `/` - Search issues
- `f` - Open filter menu

### Issue Detail View

- `e` - Edit issue
- `c` - Close issue
- `r` - Reopen issue
- `l` - Manage labels
- `d` - View dependencies
- `Esc` - Back to list

### Dependency Tree View

- `↑`/`↓` or `j`/`k` - Navigate tree
- `Space` - Expand/collapse node
- `Enter` - Jump to issue
- `a` - Add dependency
- `x` - Remove dependency
- `Esc` - Close tree view

## Common Scenarios

### Scenario: Starting a New Feature

1. Find ready work: `bd ready --type feature`
2. Review feature requirements
3. Check dependencies and blockers
4. Mark as in_progress: `bd update <id> --status in_progress`
5. Create feature branch: `git checkout -b feature/<id>`
6. Implement and test
7. Create PR and link in issue
8. Close issue after merge

### Scenario: Bug Reported in Production

1. Create P0 bug issue immediately
2. Add reproduction steps and environment info
3. Add `production` and `critical` labels
4. Mark as in_progress
5. Investigate root cause
6. Implement and test fix
7. Deploy hotfix
8. Close bug with fix description
9. Create follow-up issues if needed

### Scenario: Blocked by External Dependency

1. Mark issue as Blocked
2. Add comment: "Waiting for API endpoint from backend team"
3. Create or reference blocking issue
4. Add dependency relationship
5. Check blocker status weekly
6. When unblocked, move to In Progress

### Scenario: Reopening Closed Issue

1. Find closed issue in list (show closed issues)
2. Press `r` to reopen
3. Add comment explaining why reopening
4. Add new label if context changed (e.g., `regression`)
5. Update priority if urgency changed
6. Mark as in_progress and fix
7. Close again when complete

### Scenario: Planning a Sprint

1. Create sprint epic: "Sprint 2026-W03"
2. Review backlog issues
3. Estimate and prioritize
4. Add selected issues as dependencies to sprint
5. Check for blockers in dependency tree
6. Assign issues to team members
7. Start sprint by marking issues in_progress
8. Track daily progress through list view
9. Close sprint epic when all work complete

## Troubleshooting

### Issue Stuck in In Progress

**Problem**: Issue has been in progress for weeks
**Solutions**:
1. Add comment with current status
2. Break into smaller sub-tasks
3. Mark as Blocked if waiting on something
4. Reassign if original assignee unavailable
5. Close and create new issues if scope changed

### Cannot Close Issue

**Problem**: Issue has open dependencies blocking closure
**Solutions**:
1. View dependency tree to find blocking issues
2. Complete blocking issues first
3. Remove incorrect dependencies
4. Close or mark dependencies as won't-fix if applicable

### Too Many Open Issues

**Problem**: Hundreds of open issues, hard to find relevant work
**Solutions**:
1. Use saved filters for common views
2. Close completed work promptly
3. Archive old issues (move to backlog priority P4)
4. Use labels for categorization
5. Bulk close obsolete issues

### Lost Track of Work

**Problem**: Can't remember what you were working on
**Solutions**:
1. Filter by assignee and status in_progress
2. Use `bd list --status in_progress --assignee <username>`
3. Review recent git commits for issue IDs
4. Check open PRs for linked issues

## See Also

- [Filtering Guide](FILTERING.md) - Advanced search and filtering
- [Keyboard Shortcuts](../KEYBOARD_SHORTCUTS.md) - Complete shortcut reference
- [Widget Reference](widgets.md) - UI component documentation
- [README](../README.md) - General usage guide
