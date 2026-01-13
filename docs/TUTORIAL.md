# Tutorial

## Tutorial flow
1. Prepare a sandbox workspace using the sample data in docs/tutorial_sample_data.
2. Tour the Issues view and learn navigation basics.
3. Create and edit issues, including status and priority updates.
4. Apply labels and explore label dimensions.
5. Manage dependencies and verify the dependency tree.
6. Save and reuse filters for repeatable views.
7. Review history and notifications before wrapping up.

## Step-by-step walkthrough
1. Create a new directory for the tutorial workspace.
2. Run bd init in that directory to create the .beads database.
3. Import the sample issues:
   bd import -i C:\Development\beads-tui\docs\tutorial_sample_data\issues.jsonl
4. Start the UI:
   beads-tui
5. In the Issues view, select an issue and press Enter to view details.
6. Press Shift+S to change status and p to change priority.
7. Press Shift+L to open the label picker and apply labels.
8. Press / to filter for status:open or a label like team:infra.
9. Switch to the Labels view and confirm label usage counts.
10. Switch to the Dependencies view and verify that dependencies render.

When finished, delete the tutorial workspace or keep it for practice.
