#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Clean up duplicate beads epics by keeping only the newest one for each category.
"""
import subprocess
import json
import re
import sys
from datetime import datetime
from collections import defaultdict

# Fix Windows console encoding
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8')
    sys.stderr.reconfigure(encoding='utf-8')

def run_bd_command(args):
    """Run a bd command and return output."""
    result = subprocess.run(['bd'] + args, capture_output=True, text=True, encoding='utf-8', errors='ignore')
    return result.stdout if result.stdout else ""

def get_all_epics():
    """Get all open epic issues."""
    output = run_bd_command(['list', '--status=open', '--limit', '0'])
    epics = []

    for line in output.strip().split('\n'):
        if '[epic]' in line and '[P1]' in line:
            # Parse line format: beads-tui-xxxx [P1] [epic] open - Title
            match = re.match(r'(beads-tui-\w+)\s+\[P\d\]\s+\[epic\]\s+\w+\s+-\s+(.+)', line)
            if match:
                epic_id = match.group(1)
                title = match.group(2).strip()
                epics.append({'id': epic_id, 'title': title})

    return epics

def get_epic_details(epic_id):
    """Get detailed information about an epic."""
    output = run_bd_command(['show', epic_id])

    details = {'id': epic_id, 'created': None, 'blocks_count': 0}

    for line in output.split('\n'):
        if line.startswith('Created:'):
            created_str = line.replace('Created:', '').strip()
            try:
                details['created'] = datetime.strptime(created_str, '%Y-%m-%d %H:%M')
            except:
                pass
        elif line.startswith('Blocks'):
            match = re.search(r'Blocks \((\d+)\):', line)
            if match:
                details['blocks_count'] = int(match.group(1))

    return details

def main():
    print("🔍 Finding all epic duplicates...")

    # Group epics by title
    epics_by_title = defaultdict(list)
    all_epics = get_all_epics()

    print(f"Found {len(all_epics)} total epics")

    for epic in all_epics:
        epics_by_title[epic['title']].append(epic['id'])

    # Find duplicates (titles with more than one epic)
    duplicates = {title: ids for title, ids in epics_by_title.items() if len(ids) > 1}

    if not duplicates:
        print("✅ No duplicates found!")
        return

    print(f"\n📋 Found {len(duplicates)} epic categories with duplicates:")
    for title, ids in duplicates.items():
        print(f"  {title}: {len(ids)} instances")

    to_close = []

    # For each category, find the newest epic with most dependencies
    for title, epic_ids in duplicates.items():
        print(f"\n🔎 Analyzing '{title}'...")

        epic_details = []
        for epic_id in epic_ids:
            details = get_epic_details(epic_id)
            epic_details.append(details)
            print(f"  {epic_id}: created={details['created']}, blocks={details['blocks_count']}")

        # Sort by blocks_count (desc) then by created time (desc)
        epic_details.sort(key=lambda x: (x['blocks_count'], x['created'] or datetime.min), reverse=True)

        keeper = epic_details[0]['id']
        duplicates_to_close = [e['id'] for e in epic_details[1:]]

        print(f"  ✓ Keeping: {keeper}")
        print(f"  ✗ Closing: {', '.join(duplicates_to_close)}")

        for dup_id in duplicates_to_close:
            to_close.append((dup_id, f"Duplicate epic - keeping {keeper} as canonical"))

    # Close all duplicates
    print(f"\n🗑️  Closing {len(to_close)} duplicate epics...")

    for epic_id, reason in to_close:
        print(f"  Closing {epic_id}...")
        subprocess.run(['bd', 'close', epic_id, '--reason', reason])

    print("\n✅ Cleanup complete!")

    # Show final stats
    print("\n📊 Final statistics:")
    subprocess.run(['bd', 'stats'])

if __name__ == '__main__':
    main()
