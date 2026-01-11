#!/usr/bin/env python3
"""
Generate beads issues from the WORKPLAN.md file.

This script parses WORKPLAN.md and generates beads CLI commands to create
all epics, features, tasks, and chores defined in the work plan.

Usage:
    # Dry run (show commands without executing)
    python generate-issues.py --dry-run

    # Execute commands (create issues)
    python generate-issues.py

    # Generate shell script instead
    python generate-issues.py --output create-issues.sh
"""

import argparse
import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Tuple


class Issue:
    """Represents a beads issue."""

    def __init__(
        self,
        title: str,
        issue_type: str,
        priority: int,
        description: str = "",
        parent: str = None,
        epic_id: str = None,
    ):
        self.title = title
        self.issue_type = issue_type  # bug, feature, task, epic, chore
        self.priority = priority
        self.description = description
        self.parent = parent
        self.epic_id = epic_id  # Store epic ID for later reference
        self.beads_id = None  # Will be set after creation

    def to_command(self, bd_path: str = "bd", prefix: str = "beads-tui") -> str:
        """Generate beads CLI command to create this issue."""
        # Don't use --prefix flag - it's already set in database config
        cmd = [bd_path, "create", self.title]
        cmd.extend(["-t", self.issue_type])
        cmd.extend(["-p", str(self.priority)])

        if self.description:
            cmd.extend(["-d", self.description])

        # Don't use --parent flag to avoid routing issues
        # Parent relationships will be added as dependencies later

        cmd.append("--json")

        return " ".join(f'"{arg}"' if " " in str(arg) else str(arg) for arg in cmd)


class WorkPlanParser:
    """Parse WORKPLAN.md and extract issues."""

    EPIC_PATTERN = r'^## Epic (\d+): (.+)$'
    FEATURE_PATTERN = r'^#### (\d+\.\d+) (.+)$'
    TASK_PATTERN = r'^\*\*(\d+\.\d+\.\d+)\*\* (.+)$'
    CHORE_PATTERN = r'^\*\*(\d+\.C\.\d+)\*\* (.+)$'
    BUG_PATTERN = r'^\*\*(\d+\.B\.\d+)\*\* (.+)$'

    def __init__(self, workplan_path: Path):
        self.workplan_path = workplan_path
        self.issues: List[Issue] = []
        self.epic_map: Dict[str, str] = {}  # epic number -> beads ID

    def parse(self) -> List[Issue]:
        """Parse the work plan and extract all issues."""
        with open(self.workplan_path, 'r', encoding='utf-8') as f:
            content = f.read()

        lines = content.split('\n')
        current_epic = None
        current_epic_num = None
        current_section = None
        current_feature_desc = []

        i = 0
        while i < len(lines):
            line = lines[i].strip()

            # Epic
            epic_match = re.match(self.EPIC_PATTERN, line)
            if epic_match:
                epic_num = epic_match.group(1)
                epic_title = epic_match.group(2)

                # Extract description (next non-empty line after "**Description:**")
                desc_lines = []
                j = i + 1
                while j < len(lines):
                    if lines[j].strip().startswith("**Description:**"):
                        j += 1
                        while j < len(lines) and lines[j].strip():
                            if not lines[j].strip().startswith("###"):
                                desc_lines.append(lines[j].strip())
                                j += 1
                            else:
                                break
                        break
                    j += 1

                description = " ".join(desc_lines)
                current_epic = Issue(
                    title=epic_title,
                    issue_type="epic",
                    priority=1,
                    description=description,
                    epic_id=epic_num,
                )
                current_epic_num = epic_num
                self.issues.append(current_epic)
                i += 1
                continue

            # Section header (Features, Tasks, Chores, Potential Bugs)
            if line.startswith("### "):
                section_name = line[4:].strip()
                current_section = section_name.lower()
                i += 1
                continue

            # Feature
            feature_match = re.match(self.FEATURE_PATTERN, line)
            if feature_match and current_section and "features" in current_section:
                feature_num = feature_match.group(1)
                feature_title = feature_match.group(2)

                # Look ahead for priority, type, and description
                priority = 2  # default
                issue_type = "feature"  # default
                desc_lines = []

                j = i + 1
                while j < len(lines) and lines[j].strip():
                    next_line = lines[j].strip()
                    if next_line.startswith("- Priority:"):
                        priority = int(next_line.split(":")[1].split("(")[0].strip())
                    elif next_line.startswith("- Type:"):
                        issue_type = next_line.split(":")[1].strip()
                    elif next_line.startswith("- Description:"):
                        desc_lines.append(next_line.split(":", 1)[1].strip())
                    elif next_line.startswith("**") or next_line.startswith("###"):
                        break
                    j += 1

                description = " ".join(desc_lines)
                issue = Issue(
                    title=feature_title,
                    issue_type=issue_type,
                    priority=priority,
                    description=description,
                    parent=f"epic-{current_epic_num}",  # Will be resolved later
                )
                self.issues.append(issue)
                i += 1
                continue

            # Task
            task_match = re.match(self.TASK_PATTERN, line)
            if task_match and current_section and "tasks" in current_section:
                task_num = task_match.group(1)
                task_title = task_match.group(2)

                issue = Issue(
                    title=task_title,
                    issue_type="task",
                    priority=2,
                    parent=f"epic-{current_epic_num}",
                )
                self.issues.append(issue)
                i += 1
                continue

            # Chore
            chore_match = re.match(self.CHORE_PATTERN, line)
            if chore_match and current_section and "chores" in current_section:
                chore_num = chore_match.group(1)
                chore_title = chore_match.group(2)

                issue = Issue(
                    title=chore_title,
                    issue_type="chore",
                    priority=3,
                    parent=f"epic-{current_epic_num}",
                )
                self.issues.append(issue)
                i += 1
                continue

            # Bug
            bug_match = re.match(self.BUG_PATTERN, line)
            if bug_match and current_section and "potential bugs" in current_section:
                bug_num = bug_match.group(1)
                bug_title = bug_match.group(2)

                issue = Issue(
                    title=bug_title,
                    issue_type="bug",
                    priority=4,  # Backlog until discovered
                    parent=f"epic-{current_epic_num}",
                )
                self.issues.append(issue)
                i += 1
                continue

            i += 1

        return self.issues


def execute_command(cmd: str, dry_run: bool = False) -> Tuple[bool, str]:
    """Execute a beads command and return the issue ID."""
    if dry_run:
        print(f"[DRY RUN] {cmd}")
        return True, f"bd-{abs(hash(cmd)) % 100000:05x}"

    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            check=True,
        )

        # Parse JSON output to get issue ID
        output = json.loads(result.stdout)
        issue_id = output.get("id", "")
        print(f"[OK] Created {issue_id}: {output.get('title', '')}")
        return True, issue_id

    except subprocess.CalledProcessError as e:
        print(f"[FAILED] Failed to create issue: {e.stderr}", file=sys.stderr)
        return False, ""
    except json.JSONDecodeError as e:
        print(f"✗ Failed to parse JSON output: {e}", file=sys.stderr)
        return False, ""


def generate_issues(
    workplan_path: Path,
    bd_path: str = "bd",
    dry_run: bool = False,
    output_file: Path = None,
):
    """Generate all issues from the work plan."""
    parser = WorkPlanParser(workplan_path)
    issues = parser.parse()

    print(f"Found {len(issues)} issues to create")
    print(f"Epics: {sum(1 for i in issues if i.issue_type == 'epic')}")
    print(f"Features: {sum(1 for i in issues if i.issue_type == 'feature')}")
    print(f"Tasks: {sum(1 for i in issues if i.issue_type == 'task')}")
    print(f"Chores: {sum(1 for i in issues if i.issue_type == 'chore')}")
    print(f"Bugs: {sum(1 for i in issues if i.issue_type == 'bug')}")
    print()

    if output_file:
        # Generate shell script
        with open(output_file, 'w') as f:
            f.write("#!/bin/bash\n")
            f.write("# Auto-generated script to create beads issues from WORKPLAN.md\n\n")
            f.write("set -e\n\n")

            epic_map = {}

            # First pass: create epics
            f.write("# Create epics\n")
            for issue in issues:
                if issue.issue_type == "epic":
                    var_name = f"EPIC_{issue.epic_id}"
                    cmd = issue.to_command(bd_path)
                    f.write(f"{var_name}=$({cmd} | jq -r '.id')\n")
                    f.write(f'echo "Created epic ${var_name}: {issue.title}"\n\n')
                    epic_map[f"epic-{issue.epic_id}"] = f"${var_name}"

            # Second pass: create child issues
            f.write("\n# Create features, tasks, chores, and bugs\n")
            child_issues = []
            for issue in issues:
                if issue.issue_type != "epic":
                    parent_var = epic_map.get(issue.parent, "") if issue.parent else ""
                    cmd = issue.to_command(bd_path)
                    var_name = f"ISSUE_{len(child_issues)}"
                    f.write(f"{var_name}=$({cmd} | jq -r '.id')\\n")
                    if parent_var:
                        child_issues.append((f"${var_name}", parent_var))
            
            # Third pass: create dependencies
            if child_issues:
                f.write("\\n# Add parent-child dependencies\\n")
                for child_var, parent_var in child_issues:
                    f.write(f'{bd_path} dep add {child_var} {parent_var}\\n')

        print(f"[OK] Generated shell script: {output_file}")
        print(f"  Run with: bash {output_file}")
        return

    # Execute commands directly
    epic_map = {}

    # First pass: create epics
    print("Creating epics...")
    for issue in issues:
        if issue.issue_type == "epic":
            cmd = issue.to_command(bd_path)
            success, issue_id = execute_command(cmd, dry_run)
            if success:
                epic_map[f"epic-{issue.epic_id}"] = issue_id
                issue.beads_id = issue_id

    print()

    # Second pass: create child issues
    print("Creating features, tasks, chores, and bugs...")
    parent_map = {}  # Store child-parent relationships for later
    for issue in issues:
        if issue.issue_type != "epic":
            # Resolve parent reference but don't use --parent flag
            parent_id = epic_map.get(issue.parent, "") if issue.parent else ""
            cmd = issue.to_command(bd_path)
            success, issue_id = execute_command(cmd, dry_run)
            if success:
                issue.beads_id = issue_id
                if parent_id:
                    parent_map[issue_id] = parent_id

    print()
    
    # Third pass: create parent-child dependencies
    if parent_map and not dry_run:
        print("Adding parent-child dependencies...")
        success_count = 0
        failed_count = 0
        for i, (child_id, parent_id) in enumerate(parent_map.items()):
            dep_cmd = f'{bd_path} dep add "{child_id}" "{parent_id}"'
            # Retry up to 3 times with increasing delays for database locks
            for retry in range(3):
                try:
                    result = subprocess.run(
                        dep_cmd,
                        shell=True,
                        capture_output=True,
                        text=True,
                        check=True,
                    )
                    if (i + 1) % 50 == 0:
                        print(f"[OK] Linked {success_count + 1} dependencies...")
                    success_count += 1
                    time.sleep(0.05)  # Small delay to avoid database lock contention
                    break
                except subprocess.CalledProcessError as e:
                    if retry < 2 and "database is locked" in e.stderr:
                        time.sleep(0.5 * (retry + 1))  # Wait longer on each retry
                        continue
                    else:
                        print(f"[FAILED] Failed to link {child_id} to {parent_id}: {e.stderr}", file=sys.stderr)
                        failed_count += 1
                        break
        print(f"\n[OK] Created {success_count} dependencies ({failed_count} failed)")
        print()
    
    print("[SUCCESS] All issues created successfully!")
    print(f"  Run 'bd list --type epic' to see all epics")
    print(f"  Run 'bd dep tree <epic-id>' to see epic structure")


def main():
    parser = argparse.ArgumentParser(
        description="Generate beads issues from WORKPLAN.md"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show commands without executing them",
    )
    parser.add_argument(
        "--bd-path",
        default="bd",
        help="Path to bd executable (default: bd)",
    )
    parser.add_argument(
        "--workplan",
        type=Path,
        default=Path("WORKPLAN.md"),
        help="Path to WORKPLAN.md (default: WORKPLAN.md)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Generate shell script instead of executing commands",
    )

    args = parser.parse_args()

    if not args.workplan.exists():
        print(f"Error: {args.workplan} not found", file=sys.stderr)
        sys.exit(1)

    generate_issues(
        workplan_path=args.workplan,
        bd_path=args.bd_path,
        dry_run=args.dry_run,
        output_file=args.output,
    )


if __name__ == "__main__":
    main()
