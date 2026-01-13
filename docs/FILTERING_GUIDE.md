# Advanced Filtering Guide

A practical guide to mastering filters in beads-tui for efficient issue management.

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Filtering](#basic-filtering)
3. [Advanced Techniques](#advanced-techniques)
4. [Real-World Workflows](#real-world-workflows)
5. [Power User Tips](#power-user-tips)
6. [Troubleshooting](#troubleshooting)

## Introduction

Filters are the most powerful feature in beads-tui for managing large issue databases. This guide teaches you how to use filters effectively to:

- Find exactly what you're looking for in seconds
- Create custom views for different workflows
- Automate common searches with saved filters
- Combine filters for complex queries

### Filter Philosophy

Think of filters as layers:
1. **View Layer** - Broad categories (All, Ready, Blocked, etc.)
2. **Column Filter Layer** - Narrow by properties (status, priority, type, labels, assignee)
3. **Search Layer** - Find specific issues by text content

Apply filters from broad to specific for best results.

## Basic Filtering

### Your First Filter

Let's find all open bugs:

1. Navigate to Issues tab (press `1`)
2. Click on the **Type** column header or use column filter
3. Select **Bug**
4. Click on the **Status** column header
5. Select **Open**

You now see all open bugs. The result count updates live as you apply filters.

### Saving Your First Filter

Now let's save this filter for reuse:

1. Press `Ctrl+S` to open save dialog
2. Type "Open Bugs" as the name
3. Type `1` for hotkey (optional)
4. Press `Enter`

Your filter is saved! Press `F1` anytime to instantly apply "Open Bugs" filter.

### Quick-Select Menu

The quick-select menu (`f` key) is your filter command center:

**Try it:**
1. Press `f` to open menu
2. See your saved "Open Bugs" filter
3. Press `1` or `Enter` to apply
4. Press `f` again to see other options

## Advanced Techniques

### Technique 1: Label Logic Mastery

Labels support both AND and OR logic. Understanding when to use each is crucial:

**Use AND Logic When:**
- Issue must have multiple characteristics
- Example: bugs that are BOTH critical AND UI-related
  ```
  Labels: bug, critical, ui
  Logic: AND
  Result: Only issues with ALL three labels
  ```

**Use OR Logic When:**
- Issue can have any of several characteristics
- Example: any frontend work (could be React, Vue, or CSS)
  ```
  Labels: react, vue, css
  Logic: OR
  Result: Issues with ANY of these labels
  ```

**Exercise:** Create a filter for "Any P0/P1 backend bug"
<details>
<summary>Solution</summary>

```
Priority: P0 (use separate filter for P1)
Type: Bug
Labels: backend, api, database
Logic: OR (backend OR api OR database)
Save as: "Critical Backend Bugs"
```
</details>

### Technique 2: Smart View Stacking

Smart views provide a base filter. Stack additional filters on top:

**Example: Ready Frontend Tasks**
1. Select **Ready** view (shows open issues with no blockers)
2. Add label filter: `frontend`
3. Add type filter: `Task`

Result: Ready-to-work frontend tasks.

**Example: My Blocked P1s**
1. Select **My Issues** view
2. Select **Blocked** status
3. Select **P1** priority

Result: Your blocked high-priority work needing attention.

### Technique 3: Search Scope Optimization

Don't search everything when you know where to look:

**Title-Only Searches** (fastest):
- Use when you know the issue title
- Example: Search "authentication" in Title scope
- Finds issues titled "Fix authentication bug", "Authentication refactor", etc.

**Description-Only Searches**:
- Use for detailed requirements or specs
- Example: Search "API endpoint" in Description scope
- Finds issues with API details in description

**Notes-Only Searches**:
- Use for progress updates and comments
- Example: Search "blocked by" in Notes scope
- Finds issues with blocking notes

**Exercise:** Find an issue with "deadline" in its notes
<details>
<summary>Solution</summary>

1. Press `/` to focus search
2. Type "deadline"
3. Press `Tab` to cycle to Notes scope
4. Press `Enter` to search
</details>

### Technique 4: Regex Power Patterns

Regular expressions unlock advanced text matching:

**Find Issues by ID Pattern:**
```
Regex: ^beads-tui-[0-9a-z]{4}$
Matches: beads-tui-a1b2, beads-tui-xyz9
Doesn't Match: other-prefix-a1b2
```

**Find Issues with Version Numbers:**
```
Regex: v?\d+\.\d+\.\d+
Matches: v1.2.3, 2.0.1, version 3.4.5
Use case: Find issues mentioning specific versions
```

**Find TODO/FIXME Comments:**
```
Regex: (TODO|FIXME|HACK)
Matches: TODO: fix this, FIXME: bug, HACK: workaround
Use case: Find technical debt markers
```

**Find Email Addresses:**
```
Regex: \b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b
Matches: user@example.com, admin@company.org
Use case: Find issues with contact information
```

**Exercise:** Find issues mentioning pull request numbers (e.g., PR #123)
<details>
<summary>Solution</summary>

```
Enable: Regex mode
Pattern: PR\s*#?\d+
Matches: PR #123, PR 456, PR#789
```
</details>

### Technique 5: Fuzzy Search for Typos

Fuzzy search is forgiving of typos and variations:

**When to Enable Fuzzy:**
- Searching for names with spelling variations
- Compensating for typos
- Finding similar words

**Example:**
```
Search: "autentication" (typo)
Fuzzy: ON
Finds: "authentication" issues
```

**Exercise:** Find "database" issues even if spelled "datbase"
<details>
<summary>Solution</summary>

1. Press `/` to focus search
2. Type "datbase"
3. Enable fuzzy mode (toggle button or keyboard shortcut)
4. Issues with "database" appear despite typo
</details>

## Real-World Workflows

### Workflow 1: Daily Standup Preparation

**Goal:** Quick view of your current work for standup

**Setup:**
1. View: My Issues
2. Status: In Progress OR Blocked
3. Sort: Updated (newest first)
4. Save as: "Standup View" with `F1` hotkey

**Usage:**
- Press `F1` at start of day
- Review what you worked on yesterday
- Identify blockers
- Plan today's focus

### Workflow 2: Bug Triage Meeting

**Goal:** Prioritize incoming bugs

**Setup:**
1. Type: Bug
2. Status: Open
3. Priority: P0 OR P1 (use multiple filters)
4. Sort: Created (oldest first)
5. Save as: "Bug Triage" with `F2` hotkey

**Usage:**
- Press `F2` before triage meeting
- Review oldest critical bugs first
- Update priorities during meeting
- Filter refreshes with new priorities

### Workflow 3: Sprint Planning

**Goal:** Find ready work for next sprint

**Setup:**
1. View: Ready
2. Priority: P1 OR P2
3. Labels: sprint-candidate
4. Sort: Priority (highest first)
5. Save as: "Sprint Backlog" with `F3` hotkey

**Usage:**
- Press `F3` during planning
- Discuss top priority ready issues
- Add to sprint by updating labels
- Re-apply filter to see updated list

### Workflow 4: Code Review Queue

**Goal:** Track PRs needing review

**Setup:**
1. Labels: needs-review, pr-open (OR logic)
2. Status: In Progress
3. Sort: Created (oldest first)
4. Save as: "Review Queue" with `F4` hotkey

**Usage:**
- Press `F4` to see review queue
- Review oldest PRs first
- Remove labels when review complete
- Queue automatically updates

### Workflow 5: Technical Debt Day

**Goal:** Find technical debt to address

**Setup:**
1. Labels: tech-debt, refactor, cleanup (OR logic)
2. Status: Open
3. Priority: P2 OR P3
4. Sort: Impact/Effort ratio
5. Save as: "Tech Debt" with `F5` hotkey

**Usage:**
- Press `F5` on tech debt days
- Pick quick wins (low effort, high impact)
- Schedule larger refactors
- Track progress over time

### Workflow 6: Blocked Work Resolution

**Goal:** Unblock stalled work

**Setup:**
1. View: Blocked
2. Assignee: (empty for all, or your name)
3. Sort: Blocked time (longest first)
4. Save as: "Unblock Queue" with `F6` hotkey

**Usage:**
- Press `F6` weekly to review blocked work
- Identify oldest blocked issues
- Chase dependencies
- Update or reassign as needed

## Power User Tips

### Tip 1: Filter Chains

Create a sequence of filters for different contexts:

```
F1: All P0s
F2: All P1s
F3: My Issues
F4: Ready Work
F5: Blocked Work
F6: Recent Updates
```

Quick mental map: F1-F2 for urgency, F3-F4 for personal work, F5-F6 for status.

### Tip 2: Temporary vs Permanent Filters

**Temporary Filters:**
- One-off searches
- Exploratory queries
- Don't save, just use once

**Permanent Filters:**
- Daily workflows
- Weekly reviews
- Team conventions

Rule of thumb: If you use it more than 3 times, save it.

### Tip 3: Filter Naming Convention

Use consistent prefixes:
- `WF:` for workflow filters (e.g., "WF: Standup")
- `VIEW:` for custom views (e.g., "VIEW: Backend")
- `TEAM:` for shared filters (e.g., "TEAM: Sprint 42")
- `TEMP:` for temporary saves (e.g., "TEMP: Q4 Review")

### Tip 4: Hotkey Organization

Organize hotkeys by frequency and category:

**F1-F4:** Most frequent (daily use)
- F1: Standup view
- F2: Current sprint
- F3: My issues
- F4: Ready work

**F5-F8:** Regular use (weekly)
- F5: Bug triage
- F6: Tech debt
- F7: Code review
- F8: Planning backlog

**F9-F12:** Occasional use (monthly or special)
- F9: Quarterly goals
- F10: Stale work review
- F11: External dependencies
- F12: Documentation tasks

### Tip 5: Clear Filter Shortcut

Get overwhelmed with filters? Press `Ctrl+X` to clear ALL filters and start fresh. This is your "reset" button.

### Tip 6: Label Autocomplete

When entering labels in column filters:
- Start typing label name
- Autocomplete suggestions appear
- Press `Tab` or `Enter` to accept
- Suggestions ranked by frequency

Faster label entry = faster filtering.

### Tip 7: Combine Search with Filters

Don't choose between search and filters - use both:

```
Example: Find P1 frontend bugs mentioning "crash"
- Priority: P1
- Type: Bug
- Labels: frontend
- Search: "crash"
```

Each layer narrows the results further.

### Tip 8: View Type as Base Filter

Always start with a view type:
- **Ready** for finding next work
- **Blocked** for unblocking
- **My Issues** for personal focus
- **Recently** for staying current
- **Stale** for cleanup

Then add column filters and search on top.

### Tip 9: Filter Export and Import

Share filters with your team:
1. Saved filters live in `~/.config/beads-tui/config.json`
2. Copy `filters` section to share
3. Team members can import by pasting into their config
4. Standardize team filters for consistency

### Tip 10: Escape Hatch - CLI Filters

When UI filters aren't enough, use CLI:
```bash
bd list --status=open --priority=P0,P1 --labels=critical
```

Useful for:
- Scripting
- Bulk operations
- Custom reports

## Troubleshooting

### Problem: Filter Returns No Results

**Checklist:**
1. Are multiple restrictive filters active?
   - Solution: Remove filters one at a time to identify conflict
2. Is search text too specific?
   - Solution: Try fuzzy search or broader terms
3. Is view filter too narrow?
   - Solution: Switch to "All" view
4. Are labels mistyped?
   - Solution: Use label autocomplete to verify

### Problem: Filter Returns Too Many Results

**Checklist:**
1. Add more column filters to narrow scope
2. Switch from OR to AND logic for labels
3. Use more specific search terms
4. Apply stricter view filter (Ready vs All)
5. Use regex for precise matching

### Problem: Saved Filter Doesn't Work

**Possible Causes:**
- Config file corruption
- Filter references non-existent labels
- Status/priority values changed

**Solution:**
1. Re-create filter manually
2. Check config file syntax
3. Verify label names match exactly

### Problem: Hotkey Not Working

**Possible Causes:**
- Hotkey already assigned
- Wrong view (hotkeys only work in Issues view)
- Modifier key pressed accidentally

**Solution:**
1. Check quick-select menu (`f`) for conflicts
2. Ensure you're in Issues tab (press `1`)
3. Try reassigning hotkey

### Problem: Performance Slow with Filters

**Causes:**
- Complex regex patterns
- Fuzzy search on large dataset
- Multiple label filters with OR logic

**Solutions:**
1. Disable fuzzy search
2. Simplify regex patterns
3. Use AND logic for labels
4. Use specific search scope (Title only)

## Practice Exercises

### Exercise 1: Find Your Next Task
Create a filter that shows your highest priority ready work.
<details>
<summary>Solution</summary>

```
View: Ready
Assignee: (your username) OR (no assignee)
Priority: P0 OR P1
Sort: Priority (highest first)
Save as: "Next Task" with F1
```
</details>

### Exercise 2: Weekly Review
Create a filter for everything you touched this week.
<details>
<summary>Solution</summary>

```
View: Recently
Assignee: (your username)
Status: (any)
Sort: Updated (newest first)
Save as: "My Week" with F2
```
</details>

### Exercise 3: Team Sprint View
Create a filter for all sprint work regardless of assignee.
<details>
<summary>Solution</summary>

```
Labels: sprint-42 (or current sprint)
Status: Open OR In Progress
Sort: Priority then Type
Save as: "Sprint 42" with F3
```
</details>

### Exercise 4: Find Stale P1 Work
Identify high-priority work that hasn't been touched in 30+ days.
<details>
<summary>Solution</summary>

```
View: Stale
Priority: P1
Status: Open OR In Progress
Sort: Updated (oldest first)
Save as: "Stale P1s" with F4
```
</details>

### Exercise 5: Cross-Team Dependencies
Find issues blocked by external teams.
<details>
<summary>Solution</summary>

```
Status: Blocked
Labels: external-dependency, blocked-by-team (OR logic)
Sort: Blocked time (longest first)
Save as: "External Blocks" with F5
```
</details>

## Next Steps

1. **Practice Daily:** Use filters as part of your daily workflow
2. **Experiment:** Try different filter combinations
3. **Refine:** Update saved filters as your workflow evolves
4. **Share:** Export and share effective filters with team
5. **Automate:** Create filters for all repetitive searches

## Additional Resources

- [Filtering Reference](FILTERING.md) - Complete filter syntax reference
- [Keyboard Shortcuts](../KEYBOARD_SHORTCUTS.md) - All shortcuts including filter keys
- [Label Guide](labels.md) - Best practices for label organization
- [Configuration](../CONFIGURATION.md) - Filter configuration options

---

**Remember:** The best filter is the one you actually use. Start simple, build complexity as needed, and always prioritize clarity over cleverness.
