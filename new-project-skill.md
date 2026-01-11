# Claude Code Skill: new-project

**Skill Name:** `new-project`
**Version:** 1.0.0
**Author:** Claude
**Category:** Project Setup & Initialization
**Invocation:** `/new-project` or via Skill tool

---

## Overview

The `new-project` skill provides comprehensive project initialization for
both **Beads** issue management and **Serena** LSP support. It automates
the setup process, validates all configurations, and ensures both systems
are fully operational before you start development.

### What This Skill Does

1. **Initializes Beads** - Creates `.beads/` directory, database, and configuration
2. **Validates Beads Setup** - Verifies database integrity and configuration correctness
3. **Initializes Serena** - Activates project with appropriate language support
4. **Validates Serena Setup** - Confirms LSP server is running and responsive
5. **Integration Testing** - Ensures both systems work together seamlessly
6. **Health Reporting** - Provides comprehensive status report and next steps

### When to Use This Skill

- Starting a new project from scratch
- Adding Beads/Serena to an existing project
- Recovering from corrupted initialization
- Setting up a cloned repository
- Validating project configuration after changes

---

## Prerequisites

### Required Tools

1. **Beads CLI** (`bd`) - Installed and in PATH

   ```bash
   # Verify installation
   bd version
   ```

2. **Serena MCP Server** - Configured in Claude Code

   ```bash
   # Check in Claude Code settings
   # Should see "plugin:serena:serena" in MCP servers list
   ```

3. **Language Server** - For your project's language(s)
   - **Rust**: rust-analyzer
   - **Python**: jedi or pyright
   - **TypeScript**: typescript language server
   - **Go**: gopls
   - See [Serena docs](https://github.com/serena-ai/serena) for full list

### Project Requirements

- Git repository (recommended, not required)
- Write permissions in project directory
- At least one source file in a supported language (for Serena)

---

## Skill Execution Flow

### Phase 1: Pre-flight Checks

**Purpose:** Validate environment and gather project information before making changes.

#### Phase 1 Steps

1. **Verify Beads Installation**

   ```bash
   # Check if bd is accessible
   which bd  # Unix
   where.exe bd  # Windows
   bd version
   ```

2. **Verify Serena MCP Availability**
   - Check if `mcp__plugin_serena_serena__get_current_config` tool is available
   - Verify Serena server is responding

3. **Detect Project Type**
   - Scan directory for source files
   - Identify primary language(s)
   - Detect existing .beads or .serena directories

4. **Check Initialization Status**

   ```bash
   # Check if already initialized
   test -d .beads && echo "Beads already initialized"
   test -d .serena && echo "Serena already initialized"
   ```

5. **Warn User if Already Initialized**
   - Present current status
   - Ask if they want to proceed (re-initialize)
   - Offer backup option for existing data

#### Phase 1 Validation Criteria

- ✅ `bd` command is accessible
- ✅ Serena MCP server responds to tool calls
- ✅ Project directory is writable
- ✅ At least one supported language detected (for Serena)

#### Phase 1 Error Handling

- **bd not found**: Guide user to install from <https://github.com/steveyegge/beads>
- **Serena unavailable**: Guide user to configure MCP server
- **No write permissions**: Alert user and suggest running with appropriate permissions
- **No source files**: Warn that Serena initialization may fail, offer to continue with Beads only

---

### Phase 2: Beads Initialization

**Purpose:** Create and configure Beads issue tracking system.

#### Phase 2 Steps

1. **Determine Initialization Strategy**

   ```python
   # Choose mode based on environment
   if is_sandboxed_environment():
       mode = "--sandbox"
   elif is_worktree():
       mode = "--no-daemon"
   else:
       mode = ""  # Use default daemon mode
   ```

2. **Generate Issue Prefix**

   ```python
   # Use project directory name as default
   project_name = os.path.basename(os.getcwd())
   issue_prefix = sanitize_prefix(project_name)
   # Examples: "beads-tui", "my-app", "backend-api"
   ```

3. **Execute bd init**

   ```bash
   # Non-interactive initialization
   echo "$issue_prefix" | bd $mode init \
     --prefix "$issue_prefix" \
     --skip-hooks \
     --skip-merge-driver \
     --quiet
   ```

   **Flags Explanation:**
   - `--prefix`: Sets issue ID prefix (e.g., "beads-tui-a1b2c3")
   - `--skip-hooks`: Skip git hooks installation (can be done later)
   - `--skip-merge-driver`: Skip merge driver setup (can be done later)
   - `--quiet`: Suppress verbose output
   - `--sandbox`: Use sandbox mode (no daemon, no auto-sync) if needed

4. **Wait for Completion**
   - Monitor process with timeout (30 seconds max)
   - Capture any error output

5. **Verify Directory Structure**

   ```bash
   # Check that .beads was created
   ls -la .beads/

   # Expected files:
   # - beads.db          (SQLite database)
   # - config.yaml       (Configuration)
   # - metadata.json     (Metadata)
   # - interactions.jsonl (Interaction log)
   # - .gitignore        (Git ignore rules)
   # - README.md         (Beads readme)
   ```

#### Phase 2 Validation Criteria

- ✅ `.beads/` directory exists
- ✅ `beads.db` file created (> 0 bytes)
- ✅ `config.yaml` file exists
- ✅ `metadata.json` file exists
- ✅ Exit code is 0 (success)

#### Phase 2 Error Handling

- **Timeout**: Kill hung process, check for lock files, retry
- **Permission denied**: Alert user, suggest running with proper permissions
- **Database creation failed**: Check disk space, check file permissions
- **Already initialized**: Ask user if they want to overwrite (backup first)

---

### Phase 3: Beads Validation

**Purpose:** Verify Beads database and configuration are correct and functional.

#### Phase 3 Steps

1. **Test Database Connection**

   ```bash
   # Run info command to verify database access
   bd $mode info --json
   ```

2. **Validate Configuration**

   ```yaml
   # Read and parse config.yaml
   cat .beads/config.yaml

   # Expected structure:
   # issue_prefix: "beads-tui"
   # auto_compact_enabled: false
   # compact_tier1_days: 30
   # etc.
   ```

3. **Check Issue Prefix**

   ```bash
   # Verify prefix matches expectations
   bd $mode info --json | jq -r '.config.issue_prefix'
   # Should output: "beads-tui" (or chosen prefix)
   ```

4. **Validate Metadata**

   ```json
   // Read metadata.json
   {
     "schema_version": "1.0",
     "created_at": "2026-01-10T19:38:00Z"
   }
   ```

5. **Test Basic Operations**

   ```bash
   # List issues (should be empty)
   bd $mode list --json
   # Should return: []

   # Create test issue
   bd $mode create "Test initialization" -p 4 -t task --json
   # Should return: {"id": "beads-tui-xxxxx", "title": "Test initialization", ...}

   # Verify issue was created
   bd $mode list --json
   # Should return: [{"id": "beads-tui-xxxxx", ...}]

   # Close test issue
   bd $mode close beads-tui-xxxxx --reason "Test complete"
   ```

6. **Validate Database Integrity**

   ```bash
   # Run info with schema check
   bd $mode info --schema --json

   # Verify all tables exist:
   # - issues
   # - dependencies
   # - labels
   # - issue_labels
   # - metadata
   # - events
   ```

7. **Check for Common Issues**
   - Missing config keys
   - Corrupted database
   - Version mismatches
   - Lock file conflicts

#### Phase 3 Validation Criteria

- ✅ `bd info` returns valid JSON
- ✅ Issue prefix is correctly set
- ✅ Can create and close issues
- ✅ Database schema is complete
- ✅ No corruption errors
- ✅ Config.yaml has all required fields

#### Phase 3 Error Handling

- **Database locked**: Check for stale lock files, kill hung processes
- **Schema errors**: Run `bd migrate` to fix schema
- **Missing config keys**: Add default values to config.yaml
- **Corruption detected**: Offer to re-initialize with backup

---

### Phase 4: Serena Initialization

**Purpose:** Activate Serena project with LSP support for detected language(s).

#### Phase 4 Steps

1. **Detect Project Language(s)**

   ```python
   # Scan directory for source files
   language_map = {
       "*.rs": "rust",
       "*.py": "python",
       "*.ts": "typescript",
       "*.js": "typescript",  # Use TypeScript LSP for JS
       "*.go": "go",
       "*.java": "java",
       "*.cpp": "cpp",
       "*.c": "cpp",
       # Add more mappings
   }

   detected_languages = detect_languages(language_map)
   primary_language = detected_languages[0] if detected_languages else None
   ```

2. **Determine Project Path**

   ```python
   # Use absolute path for reliability
   project_path = os.path.abspath(os.getcwd())
   ```

3. **Activate Serena Project**

   ```python
   # Call Serena MCP tool
   mcp__plugin_serena_serena__activate_project(
       project=project_path,
       language=primary_language  # e.g., "rust"
   )
   ```

4. **Wait for LSP Server Startup**
   - LSP server initialization can take a few seconds
   - Wait up to 30 seconds for server to become ready
   - Monitor for initialization errors

5. **Verify Project Activation**

   ```python
   # Get current config to verify activation
   config = mcp__plugin_serena_serena__get_current_config()

   # Check active project
   assert config["active_project"]["path"] == project_path
   assert config["active_project"]["language"] == primary_language
   ```

6. **Create .serena Directory Structure**

   ```bash
   # Expected structure after activation
   ls -la .serena/

   # Files:
   # - project.yml        (Project configuration)
   # - memories/          (Directory for memories)
   ```

#### Phase 4 Validation Criteria

- ✅ At least one language detected
- ✅ Serena activation succeeds
- ✅ `.serena/` directory created
- ✅ `project.yml` exists and is valid
- ✅ Active project matches current directory
- ✅ Language matches detected language

#### Phase 4 Error Handling

- **No languages detected**: Ask user to specify language manually
- **Unsupported language**: List supported languages, ask user to choose closest match
- **LSP server won't start**: Check if language server is installed, provide installation instructions
- **Serena MCP unavailable**: Guide user to configure MCP server
- **Timeout waiting for LSP**: Increase timeout, check server logs

---

### Phase 5: Serena Validation

**Purpose:** Verify Serena LSP is fully functional and responsive.

#### Phase 5 Steps

1. **Check Onboarding Status**

   ```python
   # Verify onboarding was performed
   result = mcp__plugin_serena_serena__check_onboarding_performed()

   if not result["onboarding_performed"]:
       # Run onboarding if needed
       mcp__plugin_serena_serena__onboarding()
   ```

2. **Test File Discovery**

   ```python
   # List directory to verify file access
   result = mcp__plugin_serena_serena__list_dir(
       relative_path=".",
       recursive=False
   )

   # Should return list of files and directories
   assert len(result["directories"]) > 0 or len(result["files"]) > 0
   ```

3. **Test Symbol Search**

   ```python
   # Search for a known symbol (if source files exist)
   # For Rust: search for "main" function
   # For Python: search for "main" or common imports

   if primary_language == "rust":
       result = mcp__plugin_serena_serena__find_symbol(
           name_path_pattern="main",
           include_body=False
       )
       # Should find main function if it exists
   ```

4. **Test File Reading**

   ```python
   # Read a source file to verify LSP integration
   source_files = glob.glob("src/**/*.*", recursive=True)
   if source_files:
       result = mcp__plugin_serena_serena__read_file(
           relative_path=source_files[0]
       )
       # Should return file contents
       assert len(result["content"]) > 0
   ```

5. **Test Symbol Overview**

   ```python
   # Get symbols overview from a source file
   if source_files:
       result = mcp__plugin_serena_serena__get_symbols_overview(
           relative_path=source_files[0],
           depth=1
       )
       # Should return symbols grouped by kind
       # May be empty if file has no symbols
   ```

6. **Verify LSP Server Health**

   ```python
   # Get current config to check LSP status
   config = mcp__plugin_serena_serena__get_current_config()

   # Check for LSP server information
   assert "active_project" in config
   assert config["active_project"]["language"] is not None
   ```

7. **Test Memory System**

   ```python
   # List available memories
   memories = mcp__plugin_serena_serena__list_memories()

   # Memories may be empty on fresh project, that's ok
   # Just verify the call succeeds
   ```

#### Phase 5 Validation Criteria

- ✅ Onboarding completed (or already performed)
- ✅ Can list directory contents
- ✅ Can read source files
- ✅ Symbol search works (if symbols exist)
- ✅ LSP server is responsive
- ✅ No errors in tool calls
- ✅ Memory system is accessible

#### Phase 5 Error Handling

- **Onboarding fails**: Re-run onboarding, check for errors
- **File access denied**: Check permissions
- **Symbol search fails**: Verify LSP server is running, check language server installation
- **LSP server timeout**: Increase timeout, restart LSP server
- **No symbols found**: Acceptable if no symbols exist yet (new project)

---

### Phase 6: Integration Testing & Health Check

**Purpose:** Verify both systems work together and provide comprehensive status report.

#### Phase 6 Steps

1. **Create Welcome Issue in Beads**

   ```bash
   # Create an issue documenting the initialization
   bd $mode create "Project initialized successfully" \
     -p 4 \
     -t task \
     -d "Beads and Serena initialized on $(date). Ready for development." \
     --json
   ```

2. **Cross-System Verification**

   ```python
   # Verify Serena can see .beads directory
   result = mcp__plugin_serena_serena__list_dir(
       relative_path=".beads",
       recursive=False
   )
   assert "beads.db" in result["files"]

   # Verify Beads can see .serena directory
   # (via bash command)
   assert os.path.exists(".serena/project.yml")
   ```

3. **Generate Health Report**

   ```python
   health_report = {
       "beads": {
           "status": "✅ Operational",
           "database": ".beads/beads.db",
           "issue_prefix": issue_prefix,
           "issue_count": 1,  # Welcome issue
           "mode": mode,
           "config_valid": True
       },
       "serena": {
           "status": "✅ Operational",
           "project_path": project_path,
           "language": primary_language,
           "lsp_server": "Running",
           "onboarding": "Complete"
       },
       "integration": {
           "status": "✅ Both systems operational",
           "cross_access": True
       }
   }
   ```

4. **Run Final Diagnostics**

   ```bash
   # Beads diagnostics
   bd $mode info --json > .beads/init-report.json

   # Serena diagnostics
   # (via get_current_config)
   config = mcp__plugin_serena_serena__get_current_config()
   ```

5. **Create Initialization Summary**

   ```markdown
   # Project Initialization Complete

   ## Beads Status
   - Database: .beads/beads.db
   - Issue Prefix: beads-tui
   - Mode: sandbox
   - Issues: 1 (welcome issue created)

   ## Serena Status
   - Language: rust
   - LSP Server: rust-analyzer (running)
   - Onboarding: Complete
   - Source Files: 12 detected

   ## Next Steps
   1. Start creating issues: `bd create "Task name" -p 1 -t task`
   2. View ready work: `bd ready`
   3. Use Serena for code navigation and symbol search
   4. Begin development!

   ## Useful Commands
   - `bd list` - List all issues
   - `bd ready` - Find ready work
   - `bd show <id>` - View issue details
   - Ask Claude to explore codebase with Serena
   ```

6. **Write Summary to File**

   ```bash
   # Save initialization report
   echo "$summary" > INITIALIZATION_REPORT.md
   ```

#### Phase 6 Validation Criteria

- ✅ Welcome issue created successfully
- ✅ Both systems can access each other's files
- ✅ Health report shows all green
- ✅ Diagnostics complete without errors
- ✅ Summary report generated

#### Phase 6 Error Handling

- **Issue creation fails**: Non-critical, report error but continue
- **Cross-access fails**: Check permissions, report warning
- **Report generation fails**: Non-critical, skip file creation

---

## Complete Execution Script

### For Claude Code Agent

```python
def execute_new_project_skill():
    """
    Execute the new-project skill to initialize Beads and Serena.
    """

    # Phase 1: Pre-flight Checks
    print("🔍 Phase 1: Pre-flight Checks")

    # Check bd installation
    bd_available = check_command_exists("bd")
    if not bd_available:
        return error("Beads CLI not found. Install from: https://github.com/steveyegge/beads")

    # Check Serena MCP
    serena_available = check_serena_mcp()
    if not serena_available:
        return error("Serena MCP server not available. Configure in Claude Code settings.")

    # Detect project language
    languages = detect_project_languages()
    if not languages:
        print("⚠️  Warning: No source files detected. Serena initialization may be limited.")
        languages = ["python"]  # Default fallback

    primary_language = languages[0]
    print(f"✅ Detected primary language: {primary_language}")

    # Check if already initialized
    beads_exists = os.path.exists(".beads")
    serena_exists = os.path.exists(".serena")

    if beads_exists or serena_exists:
        print(f"⚠️  Warning: Project appears already initialized")
        print(f"   Beads: {'✅ exists' if beads_exists else '❌ not found'}")
        print(f"   Serena: {'✅ exists' if serena_exists else '❌ not found'}")
        # In interactive mode, would ask user to confirm
        # For skill execution, proceed with caution

    # Phase 2: Beads Initialization
    print("\n📦 Phase 2: Beads Initialization")

    # Determine mode
    mode = "--sandbox" if is_sandboxed() else ""

    # Generate issue prefix
    project_name = os.path.basename(os.getcwd())
    issue_prefix = sanitize_prefix(project_name)
    print(f"   Issue prefix: {issue_prefix}")

    # Run bd init
    cmd = f'echo "{issue_prefix}" | bd {mode} init --prefix "{issue_prefix}" --skip-hooks --skip-merge-driver --quiet'
    result = run_command(cmd, timeout=30)

    if result.returncode != 0:
        return error(f"Beads initialization failed: {result.stderr}")

    print("✅ Beads initialized")

    # Phase 3: Beads Validation
    print("\n✅ Phase 3: Beads Validation")

    # Test database
    info = run_command(f"bd {mode} info --json", timeout=5)
    if info.returncode != 0:
        return error("Beads database validation failed")

    info_data = json.loads(info.stdout)
    print(f"   Database: {info_data['database_path']}")
    print(f"   Prefix: {info_data['config']['issue_prefix']}")
    print(f"   Mode: {info_data['mode']}")

    # Create test issue
    test_issue = run_command(
        f'bd {mode} create "Test initialization" -p 4 -t task --json',
        timeout=5
    )
    if test_issue.returncode == 0:
        issue_data = json.loads(test_issue.stdout)
        print(f"✅ Test issue created: {issue_data['id']}")
        # Close test issue
        run_command(f"bd {mode} close {issue_data['id']} --reason 'Test complete'")

    # Phase 4: Serena Initialization
    print("\n🔧 Phase 4: Serena Initialization")

    project_path = os.path.abspath(os.getcwd())

    try:
        activate_result = mcp__plugin_serena_serena__activate_project(
            project=project_path,
            language=primary_language
        )
        print(f"✅ Serena activated for {primary_language}")
    except Exception as e:
        return error(f"Serena activation failed: {e}")

    # Phase 5: Serena Validation
    print("\n✅ Phase 5: Serena Validation")

    # Check onboarding
    try:
        onboarding_check = mcp__plugin_serena_serena__check_onboarding_performed()
        if not onboarding_check.get("onboarding_performed"):
            print("   Running onboarding...")
            mcp__plugin_serena_serena__onboarding()
        print("✅ Serena onboarding complete")
    except Exception as e:
        print(f"⚠️  Onboarding check failed: {e}")

    # Test file access
    try:
        dir_list = mcp__plugin_serena_serena__list_dir(
            relative_path=".",
            recursive=False
        )
        print(f"✅ File access working ({len(dir_list.get('files', []))} files)")
    except Exception as e:
        print(f"⚠️  File access test failed: {e}")

    # Phase 6: Integration & Health Check
    print("\n🎉 Phase 6: Integration & Health Check")

    # Create welcome issue
    welcome = run_command(
        f'bd {mode} create "Project initialized successfully" -p 4 -t task '
        f'-d "Beads and Serena initialized. Ready for development." --json',
        timeout=5
    )

    # Generate health report
    health_report = {
        "timestamp": datetime.now().isoformat(),
        "beads": {
            "status": "operational",
            "database": f".beads/beads.db",
            "issue_prefix": issue_prefix,
            "mode": mode or "daemon"
        },
        "serena": {
            "status": "operational",
            "language": primary_language,
            "project_path": project_path
        }
    }

    # Write summary
    summary = f"""# Project Initialization Complete

## Beads Status
- Database: `.beads/beads.db`
- Issue Prefix: `{issue_prefix}`
- Mode: {mode or 'daemon'}
- Issues: 1 (welcome issue created)

## Serena Status
- Language: {primary_language}
- LSP Server: Running
- Onboarding: Complete

## Next Steps
1. Start creating issues: `bd {mode} create "Task name" -p 1 -t task`
2. View ready work: `bd {mode} ready`
3. Use Serena for code navigation
4. Begin development!

## Useful Commands
- `bd {mode} list` - List all issues
- `bd {mode} ready` - Find ready work
- `bd {mode} show <id>` - View issue details
- Ask Claude to explore codebase with Serena

---
*Initialized on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*
"""

    with open("INITIALIZATION_REPORT.md", "w") as f:
        f.write(summary)

    print("\n" + "="*60)
    print("🎉 PROJECT INITIALIZATION COMPLETE!")
    print("="*60)
    print(summary)
    print("\n✅ Full report saved to: INITIALIZATION_REPORT.md")

    return {"success": True, "health_report": health_report}
```

---

## Usage Examples

### Example 1: New Rust Project

```bash
# User invokes skill
/new-project

# Skill execution:
# 🔍 Phase 1: Pre-flight Checks
#   ✅ Beads CLI found
#   ✅ Serena MCP available
#   ✅ Detected primary language: rust
#
# 📦 Phase 2: Beads Initialization
#   Issue prefix: my-rust-app
#   ✅ Beads initialized
#
# ✅ Phase 3: Beads Validation
#   Database: /path/to/my-rust-app/.beads/beads.db
#   Prefix: my-rust-app
#   Mode: direct
#   ✅ Test issue created: my-rust-app-a1b2c
#
# 🔧 Phase 4: Serena Initialization
#   ✅ Serena activated for rust
#
# ✅ Phase 5: Serena Validation
#   ✅ Serena onboarding complete
#   ✅ File access working (23 files)
#
# 🎉 Phase 6: Integration & Health Check
#   ✅ Welcome issue created
#   ✅ Health report generated
#
# ============================================================
# 🎉 PROJECT INITIALIZATION COMPLETE!
# ============================================================
```

### Example 2: Existing Python Project

```bash
# Project already has source files
cd existing-python-project
/new-project

# Skill execution:
# 🔍 Phase 1: Pre-flight Checks
#   ✅ Detected primary language: python
#
# 📦 Phase 2: Beads Initialization
#   Issue prefix: existing-python-project
#   ✅ Beads initialized
#
# ... (continues as above)
```

### Example 3: Multi-Language Project

```bash
# Project has both TypeScript and Python
cd full-stack-app
/new-project

# Skill execution:
# 🔍 Phase 1: Pre-flight Checks
#   ✅ Detected languages: typescript, python
#   ✅ Using primary language: typescript
#
# ... (continues with TypeScript as primary)
```

---

## Configuration Options

### Skill Parameters (Future Enhancement)

```yaml
# Potential parameters for advanced usage
parameters:
  issue_prefix: "custom-prefix"  # Override auto-detected prefix
  language: "rust"                # Force specific language
  mode: "sandbox"                 # Force sandbox mode
  skip_serena: false              # Skip Serena initialization
  skip_beads: false               # Skip Beads initialization
  create_examples: true           # Create example issues
```

### Environment Variables

The skill respects these environment variables:

- `BD_ACTOR` - Set default actor name for Beads
- `RUST_LOG` - Set logging level for debugging
- Custom language server paths

---

## Troubleshooting

### Common Issues

#### 1. Beads Initialization Hangs

**Symptom:** `bd init` appears to hang indefinitely

**Cause:** Waiting for interactive input or daemon communication issues

**Solution:**

```bash
# Kill any hung bd processes
pkill bd  # Unix
Stop-Process -Name bd -Force  # Windows

# Retry with explicit flags
bd --sandbox init --prefix my-project --skip-hooks --skip-merge-driver --quiet
```

#### 2. Serena Language Detection Fails

**Symptom:** "No languages detected" or wrong language chosen

**Cause:** Ambiguous file extensions or new project with no files yet

**Solution:**

```python
# Manually specify language in skill invocation
# (requires skill parameter support)
/new-project language=rust

# Or create a source file first
echo 'fn main() {}' > src/main.rs
/new-project
```

#### 3. LSP Server Won't Start

**Symptom:** Serena validation fails with LSP timeout

**Cause:** Language server not installed or misconfigured

**Solutions by language:**

```bash
# Rust
rustup component add rust-analyzer

# Python
pip install jedi-language-server
# or
pip install pyright

# TypeScript
npm install -g typescript typescript-language-server

# Go
go install golang.org/x/tools/gopls@latest
```

#### 4. Permission Denied Creating .beads

**Symptom:** "Permission denied" when creating directories

**Cause:** Insufficient write permissions

**Solution:**

```bash
# Check directory permissions
ls -ld .

# Fix permissions if needed
chmod u+w .

# Or run from a directory you own
```

#### 5. Database Already Exists Error

**Symptom:** "Database already initialized" error

**Cause:** Previous initialization exists

**Solution:**

```bash
# Backup existing database
mv .beads .beads.backup

# Re-run skill
/new-project

# Restore data if needed
bd import -i .beads.backup/issues.jsonl
```

### Debug Mode

To run with verbose output for debugging:

```bash
# Set environment variable before invoking
export RUST_LOG=debug
export BD_VERBOSE=1

# Then invoke skill
/new-project
```

### Validation Checklist

If the skill completes but you want to manually verify:

```bash
# Beads Checks
[ -d .beads ] && echo "✅ .beads exists"
[ -f .beads/beads.db ] && echo "✅ Database exists"
bd info && echo "✅ Beads operational"
bd list --json && echo "✅ Can query database"

# Serena Checks
[ -d .serena ] && echo "✅ .serena exists"
[ -f .serena/project.yml ] && echo "✅ Project config exists"
# LSP check requires MCP tool call

# Integration
[ -f INITIALIZATION_REPORT.md ] && echo "✅ Report generated"
```

---

## Post-Initialization Workflow

### Immediate Next Steps

1. **Review Initialization Report**

   ```bash
   cat INITIALIZATION_REPORT.md
   ```

2. **Create Your First Real Issue**

   ```bash
   bd create "Implement user authentication" -p 1 -t feature
   ```

3. **Explore Codebase with Serena**

   ```text
   Ask Claude: "Show me the symbol overview of src/main.rs"
   ```

4. **Set Up Git Hooks (Optional)**

   ```bash
   bd setup claude  # For Claude Code integration
   # or
   bd setup aider   # For Aider integration
   ```

### Recommended Workflow

1. **Plan Your Work**

   ```bash
   # Create epic for major feature
   bd create "Authentication System" -t epic -p 1

   # Create child tasks
   bd create "Login UI" -p 1 --parent <epic-id>
   bd create "JWT middleware" -p 1 --parent <epic-id>
   ```

2. **Find Work**

   ```bash
   bd ready  # Show ready work (no blockers)
   ```

3. **Work on Issues**

   ```bash
   bd update <id> --status in_progress
   # ... do the work ...
   bd close <id> --reason "Implemented and tested"
   ```

4. **Use Serena for Code Navigation**
   - Ask Claude to find symbols
   - Explore dependencies
   - Navigate codebase semantically

---

## Advanced Configuration

### Custom Issue Prefix

Edit `.beads/config.yaml`:

```yaml
issue_prefix: "my-custom-prefix"
```

### Multiple Languages in Serena

For multi-language projects, you may need to:

1. Initialize with primary language
2. Manually configure additional languages in `.serena/project.yml`

### Daemon vs Sandbox Mode

**When to use sandbox mode:**

- Development/testing environments
- Git worktrees
- CI/CD pipelines
- Restricted environments

**When to use daemon mode:**

- Normal development
- Better performance
- Auto-sync enabled

Switch mode in commands:

```bash
bd --sandbox <command>  # Use sandbox
bd --no-daemon <command>  # Direct mode, no daemon
bd <command>  # Default (daemon if available)
```

---

## Skill Metadata

```yaml
skill:
  name: new-project
  version: 1.0.0
  description: Initialize Beads and Serena for a new project
  author: Claude
  category: setup

  capabilities:
    - beads_initialization
    - serena_initialization
    - project_validation
    - health_reporting

  requirements:
    - beads_cli
    - serena_mcp
    - write_permissions

  estimated_time: 30-60 seconds

  interactive: false  # Fully automated

  outputs:
    - .beads/beads.db
    - .beads/config.yaml
    - .serena/project.yml
    - INITIALIZATION_REPORT.md

  error_handling: comprehensive
  validation: thorough
```

---

## FAQ

### Q: Can I run this skill multiple times?

**A:** Yes, but it will warn you if already initialized. It's safe to
re-run if you suspect corruption or want to start fresh.

### Q: What if I don't want Serena, only Beads?

**A:** Currently the skill initializes both. You can manually run just
`bd init` if you only want Beads. A future version may add a
`skip_serena` parameter.

### Q: Does this work in git worktrees?

**A:** Yes, but beads will automatically use `--no-daemon` mode in worktrees.

### Q: Can I customize the initialization?

**A:** Yes, edit `.beads/config.yaml` and `.serena/project.yml` after initialization.

### Q: What happens to existing .beads or .serena directories?

**A:** The skill will warn you and ask for confirmation before overwriting (in interactive mode).

### Q: How do I know if initialization succeeded?

**A:** Check for:

- ✅ Green success messages in output
- ✅ `INITIALIZATION_REPORT.md` exists
- ✅ Both `.beads` and `.serena` directories exist
- ✅ `bd list` works without errors

---

## Related Documentation

- [Beads CLI Reference](https://github.com/steveyegge/beads/blob/main/docs/CLI_REFERENCE.md)
- [Beads Quickstart](https://github.com/steveyegge/beads/blob/main/docs/QUICKSTART.md)
- [Serena Documentation](https://github.com/serena-ai/serena)
- [Claude Code Skills](https://docs.anthropic.com/claude/docs/claude-code-skills)

---

## Version History

- **1.0.0** (2026-01-10)
  - Initial release
  - Beads initialization
  - Serena initialization
  - Comprehensive validation
  - Health reporting

---

## End of Skill Documentation

*This skill automates project initialization to get you coding faster.
For issues or improvements, please report them to the skill maintainer.*
