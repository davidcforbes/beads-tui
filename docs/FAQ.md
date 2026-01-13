# FAQ

## Do I need the beads CLI installed?
Yes. Beads-TUI shells out to the beads CLI for reads and writes.

## Where is my data stored?
Beads stores data in the .beads directory at the root of your workspace.
Beads-TUI reads and updates issues through that database.

## Why do I not see any issues?
Make sure you are running beads-tui inside a workspace that has been
initialized with bd init and contains issues.

## How do I refresh data?
Press r to reload data from the beads CLI. You can also close and reopen
beads-tui if the daemon was restarted.

## How do I change status or priority?
Select an issue, then press Shift+S for status or p for priority.

## How do I edit labels?
Select an issue and press Shift+L to open the label picker.

## Where are keyboard shortcuts listed?
See KEYBOARD_SHORTCUTS.md or press ? inside the UI.

## What if the UI looks broken?
Try resizing the terminal, using a 256 color terminal, or restarting.
If the problem persists, check logs with RUST_LOG=debug.
