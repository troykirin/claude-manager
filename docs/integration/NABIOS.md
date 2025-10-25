# NabiÓS Integration (Optional)

**Note:** This is an optional advanced feature for users running multi-agent federation systems.

## What is NabiÓS?

[NabiÓS](https://github.com/yourusername/nabios) is a federated agent coordination platform that enables:
- Multi-agent collaboration across distributed systems
- Centralized knowledge management
- Coordinated tool execution across Claude instances
- Session synchronization between nodes

## When to Use NabiÓS Integration

Consider NabiÓS integration if you:
- Run Claude Code on multiple machines (macOS, WSL, Linux, Raspberry Pi)
- Need session synchronization across environments
- Use federated agent architectures
- Want centralized session analytics and monitoring

## Standalone vs Federation Mode

**claude-manager** works perfectly as a standalone tool. NabiÓS integration is **entirely optional** and adds:

| Feature | Standalone | With NabiÓS |
|---------|-----------|-------------|
| Local migration | ✅ | ✅ |
| Session backup | ✅ | ✅ |
| Undo support | ✅ | ✅ |
| Multi-node sync | ❌ | ✅ |
| Federation events | ❌ | ✅ |
| Coordinated agents | ❌ | ✅ |

## Installation

### Standalone (Default)

```bash
# Standard installation - no federation
./install.sh
```

### With NabiÓS Federation

```bash
# Install NabiÓS platform first
git clone https://github.com/yourusername/nabios.git
cd nabios && ./install.sh

# Install claude-manager with federation support
git clone https://github.com/yourusername/claude-manager.git
cd claude-manager
./install.sh --with-federation
```

## Configuration

### Standalone Configuration

```bash
# ~/.claude-manager.conf (default)
export CLAUDE_DIR="$HOME/.claude"
export CLAUDE_BACKUP_STRATEGY="file"
export CLAUDE_INTERACTIVE="true"
```

### Federation Configuration

```bash
# ~/.claude-manager.conf (with NabiÓS)
export CLAUDE_DIR="$HOME/.claude"
export CLAUDE_BACKUP_STRATEGY="file"
export CLAUDE_INTERACTIVE="true"

# Federation settings
export NABIOS_ENABLED="true"
export NABIOS_FEDERATION_URL="http://localhost:5000"
export NABIOS_AGENT_ID="$(cat ~/.config/nabi/agent-id)"
```

## Federation Features

### 1. Multi-Node Session Sync

When you migrate a project on one machine, NabiÓS can coordinate updates across all nodes:

```bash
# On machine A (macOS)
cm move --federated "/Users/tryk/dev/project" "/Users/tryk/projects/project"

# Sessions automatically updated on:
# - Machine B (WSL)
# - Machine C (Linux server)
# - Machine D (Raspberry Pi)
```

### 2. Federated Event Logging

Migration operations emit federation events for monitoring:

```bash
# View migration events across all nodes
nabi events --filter "claude-manager:migrate"

# Check migration status
nabi status claude-manager
```

### 3. Coordinated Backup Strategy

Federation-wide backup coordination:

```bash
# Backup across all nodes before migration
cm move --federated --backup-all "/old" "/new"
```

## Example: Federated Migration

```bash
# 1. Check federation status
nabi status

# 2. Preview migration across all nodes
CLAUDE_DRY_RUN=true cm move --federated "/old" "/new"

# 3. Execute coordinated migration
cm move --federated "/old" "/new"

# 4. Verify on all nodes
nabi query "claude-manager:sessions" --node all
```

## Federation Architecture

```
┌─────────────────────────────────────────────────────┐
│ NabiÓS Federation Layer                             │
│ • Event Bus (Loki)                                  │
│ • Coordination Server (NATS/Redis)                  │
│ • Knowledge Graph (SurrealDB)                       │
└─────────────────────────────────────────────────────┘
                         ↓
         ┌───────────────┼───────────────┐
         ↓               ↓               ↓
    ┌────────┐      ┌────────┐      ┌────────┐
    │ Node A │      │ Node B │      │ Node C │
    │ macOS  │      │  WSL   │      │  RPi   │
    └────────┘      └────────┘      └────────┘
         ↓               ↓               ↓
  claude-manager  claude-manager  claude-manager
```

## Disabling Federation

If you installed with federation support but want to disable it:

```bash
# Temporary (single command)
NABIOS_ENABLED=false cm move "/old" "/new"

# Permanent (edit config)
# ~/.claude-manager.conf
export NABIOS_ENABLED="false"
```

## When NOT to Use Federation

Federation adds complexity. Skip it if you:
- Only use Claude Code on one machine
- Don't need session synchronization
- Want simpler setup and maintenance
- Prefer standalone tools

**The standalone version is powerful and complete for most users.**

## Future Federation Features

Planned enhancements:

- **Cross-node session migration** - Move sessions between machines
- **Federated session analytics** - Aggregate stats across all nodes
- **Conflict resolution** - Automatic merge of divergent sessions
- **Agent coordination** - Multi-agent project migrations

## Links

- [NabiÓS Platform](https://github.com/yourusername/nabios)
- [Federation Protocol](https://github.com/yourusername/nabios/blob/main/docs/FEDERATION_PROTOCOL.md)
- [claude-manager Standalone](../usage/USAGE.md)

---

**Remember:** Federation is optional. claude-manager is fully functional standalone!
