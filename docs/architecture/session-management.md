# Claude Session State Architecture - Reverse Engineering Analysis

## Executive Summary

Through extensive reverse engineering, we have discovered that Claude's session management is a complex distributed system with multiple interconnected data stores. This document provides the first comprehensive technical analysis of Claude's internal state architecture and critical corruption patterns that can render sessions unrecoverable.

## Core Architecture Overview

### Data Store Distribution

Claude maintains session state across four primary systems:

```
~/.claude/
├── projects/           # Primary session content (JSONL format)
├── todos/             # Active session state & agent coordination  
├── statsig/           # Analytics & session tracking metadata
└── shell-snapshots/   # Environment state preservation
```

### Session Identification System

- **Primary Key**: Session UUID (e.g., `6fd63673-de16-46f0-bf8b-20e667da9657`)
- **Cross-Reference Pattern**: Same UUID appears across all four systems
- **Directory Naming**: Project directories encoded by original startup path
  - Format: `-Users-tryk-nabia-claude-manager`
  - Encoding: Original path with slashes replaced by dashes

### Data Store Relationships

#### 1. Projects Directory (`~/.claude/projects/`)
- **Purpose**: Primary conversation storage
- **Format**: JSONL (JSON Lines) files
- **Content**: Message history, tool calls, metadata
- **Naming**: `{session-uuid}.jsonl`
- **Critical Fields**:
  ```json
  {
    "sessionId": "6fd63673-de16-46f0-bf8b-20e667da9657",
    "cwd": "/Users/tryk/nabia/claude-manager",
    "timestamp": "2025-09-04T06:58:53.999Z"
  }
  ```

#### 2. Todos Directory (`~/.claude/todos/`)
- **Purpose**: Active task state and agent coordination
- **Format**: JSON files
- **Content**: Todo lists, agent state, task coordination
- **Naming**: `{session-uuid}-agent-{agent-uuid}.json`
- **Volume**: 325+ active files discovered in typical installation

#### 3. Statsig Directory (`~/.claude/statsig/`)
- **Purpose**: Analytics and feature flag evaluation
- **Files**:
  - `statsig.cached.evaluations.*`
  - `statsig.session_id.*`
  - `statsig.stable_id.*`

#### 4. Shell Snapshots (`~/.claude/shell-snapshots/`)
- **Purpose**: Environment state preservation
- **Format**: Shell script snapshots
- **Naming**: `snapshot-zsh-{timestamp}-{random}.sh`
- **Volume**: 87+ files in typical installation

## Session Lifecycle States

### 1. Active Session
- Process running (`ps aux | grep claude`)
- Todo files present and recently modified
- Recent shell snapshots
- Statsig session active

### 2. Suspended Session  
- No active process
- Todo files preserved
- Can be resumed cleanly
- State consistent across systems

### 3. Archived Session
- Project JSONL files remain
- Todo files cleaned up
- Shell snapshots stale
- Session effectively closed

### 4. Corrupted Session
- **CRITICAL**: Inconsistent state across systems
- Cannot be resumed normally
- History navigation fails (double-tap ESC broken)
- Requires recovery tooling

## Critical Discovery: Path-Based Project Organization

Project directories are organized by the **original path** where the session was started:

```bash
# Session started in: /Users/tryk/nabia/claude-manager  
# Results in directory: ~/.claude/projects/-Users-tryk-nabia-claude-manager/

# Path encoding algorithm:
# 1. Take original startup path
# 2. Replace '/' with '-' 
# 3. Prefix with single '-'
```

This explains why migration is complex - the directory name itself encodes the original working directory.

## State Consistency Requirements

For a session to be valid and resumable:

1. **Session UUID Consistency**: Same UUID must exist across all systems
2. **Path Consistency**: `cwd` field in JSONL must match expected path
3. **Temporal Consistency**: Modification times should be logically consistent
4. **Process Consistency**: No competing processes for same session UUID

## Technical Implications

### Migration Complexity
- Cannot simply move JSONL files
- Must update `cwd` fields within JSONL content
- Must handle cross-system references
- Directory name itself must change to reflect new path

### Branching/Splitting Risks
- Creating branches while original session active = corruption
- Multiple processes competing for same session UUID
- Cross-system state becomes inconsistent
- Recovery requires specialized tooling

### Scale Implications
- 325+ todo files indicate significant state management overhead
- 87+ shell snapshots suggest heavy environment tracking
- Multiple statsig files per user session
- Cross-system consistency checks become expensive at scale

## Process Detection Patterns

In our analysis, we discovered 30+ active Claude processes:
- Multiple terminal sessions
- Background agent processes  
- Zed editor integrations
- MCP server connections

Each process can potentially modify session state, creating race conditions during migration.

## Next Steps for Production Tooling

1. **State Validation Engine**: Check cross-system consistency
2. **Process Management**: Safe shutdown protocols
3. **Atomic Migration**: Transaction-like operations across all systems
4. **Recovery Tooling**: Handle corrupted state scenarios
5. **Monitoring**: Detect inconsistencies before they cause corruption

---

*This analysis represents the first comprehensive reverse-engineering of Claude's internal session state architecture. These findings are critical for building production-ready session management tooling.*