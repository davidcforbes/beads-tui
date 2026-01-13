# Workflow Examples

## Triage new issues
1. Open the Issues view.
2. Press c to create a new issue, fill out the form, then press Ctrl+S.
3. Use / to focus the search bar and filter by text or labels.
4. Use Shift+S to set status and p to set priority.
5. Use Shift+L to apply labels and keep the list consistent.

## Review blocked work
1. Use / to search for "status:blocked" or filter by labels like state:blocked.
2. Sort by priority to handle P0 and P1 issues first.
3. Open an issue to inspect dependencies and add notes.

## Manage dependencies
1. Switch to the Dependencies view.
2. Select an issue and open the dependency dialog to add or remove links.
3. Review the dependency tree to confirm no cycles were introduced.
4. Return to the Issues view and verify status updates.

## Label cleanup and taxonomy
1. Switch to the Labels view to browse label usage.
2. Use the search bar to find duplicates or inconsistent naming.
3. Update issues using Shift+L and normalize labels to a single spelling.
4. Track dimensions using labels like state:patrol or team:infra.

## Sync and refresh
1. Press r to refresh data after making changes.
2. Use the Database view to check daemon and sync status.
3. Run bd sync in a terminal when you want to push changes.
