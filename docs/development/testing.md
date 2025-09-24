# Claude Manager - Test Scenarios & User Stories

## Overview
This document captures comprehensive test scenarios for the Claude Manager CLI tool using Gherkin-style user stories. Each scenario covers specific use cases, edge cases, and error conditions to ensure robust functionality.

---

## Feature: Session Path Migration

### Scenario: Successful Basic Migration
**Given** a Claude project directory exists at `~/.claude/projects/my-project/` with sessions
**And** those sessions contain paths pointing to `/old/workspace/project`
**And** the actual source directory has been moved to `/new/workspace/project`
**When** I run `cm migrate /old/workspace/project /new/workspace/project ~/.claude/projects/my-project/`
**Then** all session files should be updated with the new path
**And** the system should report "X files updated successfully"
**And** the original session files should be backed up with `.bak` extension

### Scenario: Migration with No Matching Sessions
**Given** a Claude project directory exists at `~/.claude/projects/my-project/`
**And** the project contains session files
**But** none of the sessions reference the specified old path `/nonexistent/path`
**When** I run `cm migrate /nonexistent/path /new/path ~/.claude/projects/my-project/`
**Then** the system should display a warning "No sessions found with path: /nonexistent/path"
**And** no migration operations should be performed
**And** no backup files should be created

### Scenario: Interactive Migration Selection
**Given** multiple Claude project directories exist
**And** I have not specified a project directory in the command
**And** interactive mode is enabled (default)
**When** I run `cm migrate /old/path /new/path`
**Then** the system should display a numbered list of available projects
**And** prompt me to "Select project (1-X):"
**And** wait for my input before proceeding

### Scenario: Migration with Auto-Project Detection
**Given** only one Claude project directory exists
**And** I have not specified a project directory in the command
**When** I run `cm migrate /old/path /new/path`
**Then** the system should automatically select the single available project
**And** display "Auto-selected project: /path/to/project"
**And** proceed with the migration without prompting

---

## Feature: Complete Project Relocation

### Scenario: Successful Source and Project Move
**Given** a source directory exists at `/workspace/project`
**And** a corresponding Claude project exists at `~/.claude/projects/project/`
**And** the Claude project contains sessions referencing `/workspace/project`
**When** I run `cm move /workspace/project /workspace/new-project ~/.claude/projects/project/ ~/.claude/projects/new-project/`
**Then** the source directory should be moved to `/workspace/new-project`
**And** the Claude project should be moved to `~/.claude/projects/new-project/`
**And** all session paths should be updated to reference `/workspace/new-project`
**And** the system should report successful completion

### Scenario: Move with Destination Conflicts
**Given** a source directory exists at `/workspace/project`
**And** a destination directory already exists at `/workspace/new-project`
**And** interactive mode is enabled
**When** I run `cm move /workspace/project /workspace/new-project`
**Then** the system should warn "Destination already exists: /workspace/new-project"
**And** prompt "Proceed and replace/merge destination? (y/N):"
**And** wait for user confirmation before proceeding

---

## Feature: Interactive Full Migration

### Scenario: Full Migration from Source Directory
**Given** I am currently in a directory `/workspace/my-project`
**And** a Claude project exists that references `/workspace/my-project`
**When** I run `cm full ../new-location`
**Then** the system should auto-detect `/workspace/my-project` as the source
**And** resolve `../new-location` relative to current directory
**And** find the corresponding Claude project automatically
**And** proceed with moving both source and project directories

### Scenario: Full Migration with Multiple Project Options
**Given** I am in a directory `/workspace/project`
**And** multiple Claude projects reference paths in `/workspace/project`
**When** I run `cm full /new/destination`
**Then** the system should display "Multiple projects reference this path:"
**And** show a numbered list of matching projects
**And** prompt for project selection before proceeding

### Scenario: Full Migration with Auto-Project Selection
**Given** I am in a directory `/workspace/project`
**And** only one Claude project references `/workspace/project`
**When** I run `cm full /new/destination`
**Then** the system should auto-select the single matching project
**And** proceed without prompting for project selection

---

## Feature: Error Handling and Safety

### Scenario: Invalid Source Directory
**Given** the specified source directory does not exist
**When** I run `cm migrate /nonexistent/source /valid/destination`
**Then** the system should display an error "Source directory not found or not a directory: /nonexistent/source"
**And** exit with error code without performing any operations

### Scenario: Non-existent Claude Directory
**Given** the `~/.claude` directory does not exist
**When** I run any cm command
**Then** the system should display "Claude directory not found: ~/.claude"
**And** provide guidance on setting up Claude or using CLAUDE_DIR variable

### Scenario: Permission Denied
**Given** I don't have write permissions for session files
**When** I attempt a migration
**Then** the system should display "Permission denied" error
**And** provide clear indication of which files couldn't be modified
**And** suggest running with appropriate permissions

### Scenario: Dry Run Mode
**Given** dry run mode is enabled with `CLAUDE_DRY_RUN=true`
**And** valid migration parameters are provided
**When** I run `cm migrate /old/path /new/path`
**Then** the system should display what would be done without making changes
**And** show preview of affected files and operations
**And** no actual file modifications should occur

---

## Feature: Configuration and Environment

### Scenario: Custom Claude Directory
**Given** I have set `CLAUDE_DIR=/custom/claude/path`
**And** my Claude data is stored in `/custom/claude/path`
**When** I run any cm command
**Then** the system should use `/custom/claude/path` instead of `~/.claude`
**And** all operations should work with the custom directory

### Scenario: Backup Strategy Selection
**Given** I have set `CLAUDE_BACKUP_STRATEGY=file`
**And** a migration operation is performed
**Then** each modified session file should be backed up as `filename.jsonl.bak`
**And** the system should report individual file backups

### Scenario: Project-Level Backup
**Given** I have set `CLAUDE_BACKUP_STRATEGY=project`
**And** a migration operation affects multiple files
**Then** the entire project directory should be backed up as a tar.gz archive
**And** the backup should be timestamped and stored safely

---

## Feature: Build and Development Tasks

### Scenario: Complete Build Process
**Given** all source code is present (Rust TUI, TypeScript federation, shell scripts)
**When** I run `task all:build`
**Then** the Rust TUI should compile successfully
**And** the TypeScript federation should build without errors
**And** the shell scripts should be validated
**And** the system should report "All components built successfully!"

### Scenario: Individual Component Building
**Given** the Rust TUI source code is present
**When** I run `task tui:build`
**Then** Cargo should compile the project with release optimizations
**And** the binary should be available in the target directory
**And** the system should report "TUI built successfully!"

### Scenario: Development Environment Setup
**Given** a clean development environment
**When** I run `task dev:setup`
**Then** the cm command should be installed
**And** the TUI should be built
**And** the system should report "Development setup complete!"

---

## Feature: System Health and Diagnostics

### Scenario: Health Check with All Components Present
**Given** Claude directory exists with projects
**And** the cm command is installed
**And** Rust toolchain is available
**When** I run `task health`
**Then** the system should report "Claude directory: Found"
**And** report "cm command: Available"
**And** report "Rust toolchain: Available"

### Scenario: Health Check with Missing Components
**Given** the Rust toolchain is not installed
**When** I run `task health`
**Then** the system should report "Rust toolchain: Not found"
**And** provide guidance on installing the required tools
**And** still report status for other components

---

## Feature: Documentation and Help

### Scenario: Main Help Display
**Given** no arguments are provided to cm
**When** I run `cm` or `cm help`
**Then** the system should display the complete usage information
**And** show all available commands with descriptions
**And** include examples and configuration options

### Scenario: Command-Specific Help
**Given** I need help with a specific command
**When** I run `cm migrate --help` (if supported)
**Then** the system should display detailed help for the migrate command
**And** show parameter descriptions and usage examples

### Scenario: Task System Help
**When** I run `task --list`
**Then** the system should display all available tasks
**And** show descriptions for each task
**And** indicate task categories (build, test, dev, etc.)

---

## Test Data Setup

### Sample Project Structure for Testing
```
~/.claude/
├── projects/
│   ├── test-project-1/
│   │   ├── session-1.jsonl (contains /old/path references)
│   │   └── session-2.jsonl (contains /old/path references)
│   └── test-project-2/
│       └── session-3.jsonl (contains /different/path references)
└── test-workspace/
    ├── old-project/ (source to be moved)
    └── new-project/ (destination for moves)
```

### Environment Variables for Testing
```bash
export CLAUDE_DIR="$HOME/.claude"
export CLAUDE_BACKUP_STRATEGY="file"
export CLAUDE_INTERACTIVE="true"
export CLAUDE_DRY_RUN="false"
```

---

## Success Criteria

### Functional Requirements
- ✅ All commands execute without errors on valid inputs
- ✅ Error messages are clear and actionable
- ✅ Interactive prompts work correctly
- ✅ File operations preserve data integrity
- ✅ Backup strategies work as expected

### Performance Requirements
- ✅ Large session files process efficiently
- ✅ Multiple project operations complete in reasonable time
- ✅ Memory usage remains bounded
- ✅ No resource leaks during operations

### Usability Requirements
- ✅ Help text is clear and comprehensive
- ✅ Error messages provide guidance for resolution
- ✅ Interactive mode is intuitive
- ✅ Progress indicators show operation status

---

## Regression Test Checklist

- [ ] Basic migration functionality
- [ ] Interactive project selection
- [ ] Error handling for missing directories
- [ ] Backup file creation and cleanup
- [ ] Dry run mode behavior
- [ ] Custom CLAUDE_DIR usage
- [ ] Permission denied scenarios
- [ ] Build system functionality
- [ ] Task automation
- [ ] Health check accuracy
