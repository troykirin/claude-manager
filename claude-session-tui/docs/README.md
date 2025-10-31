# Claude Manager Documentation

Welcome to the Claude Manager documentation. This directory contains comprehensive technical documentation for the Claude session management system.

## 📚 Documentation Structure

### [Architecture](./architecture/)
Technical architecture and system design documentation:
- [**components.md**](./architecture/components.md) - System components and their interactions
- [**session-management.md**](./architecture/session-management.md) - Claude session architecture
- [**safety-protocols.md**](./architecture/safety-protocols.md) - Production safety procedures
- [**state-corruption.md**](./architecture/state-corruption.md) - State corruption analysis and recovery

### [Development](./development/)
Developer guides and contribution documentation:
- [**testing.md**](./development/testing.md) - Test scenarios and testing strategy
- [**release-review.md**](./development/release-review.md) - Release procedures and checklist

### [Operations](./operations/)
Operational guides (to be added):
- Installation procedures
- Troubleshooting guides
- Recovery procedures

### [Archived](./archived/)
Historical documentation preserved for reference:
- [**handoff-summary.md**](./archived/handoff-summary.md) - Original project handoff documentation
- [**debugging-sessions/**](./archived/debugging-sessions/) - Historical debugging sessions
- [**analysis-reports/**](./archived/analysis-reports/) - External analysis reports

## 🚀 Quick Start

1. **Installation**: Run `task install` to install the `cm` command
2. **Build**: Run `task all:build` to build all components
3. **Test**: Run `task all:test` to verify functionality
4. **Usage**: Run `cm help` for command documentation

## 🏗️ System Overview

Claude Manager consists of three main components:

1. **Core CLI** (`claude-manager.sh`) - Bash script for session migration
2. **TUI Interface** (`claude-session-tui/`) - Rust-based terminal UI for session analysis
3. **Federation Integration** (`federation-integration/`) - TypeScript layer for multi-agent coordination

## 📋 Key Features

- **Session Path Migration**: Update Claude session paths after directory moves
- **Project Management**: Move projects between directories with automatic session updates
- **Session Analysis**: Advanced search and insights from conversation history
- **Federation Support**: Multi-agent coordination and Linear issue integration
- **Safety Protocols**: Atomic operations with backup/rollback capabilities

## 🛠️ Development

See [Taskfile.yml](../Taskfile.yml) for all available development tasks:
- `task dev:setup` - Complete development environment setup
- `task tui:dev` - Run TUI in development mode
- `task federation:dev` - Start federation development server

## 📈 Current Status

- **Architecture Grade**: B- (Solid foundation with improvement opportunities)
- **Code Quality**: A- (90/100)
- **Security Risk**: MEDIUM (Some data privacy concerns)
- **Test Coverage**: ~60% overall

See [TODO.md](../TODO.md) for current development priorities and known issues.

## 🔒 Security Considerations

- Session files may contain sensitive data (API keys, credentials)
- Implement content sanitization when processing sessions
- Consider encryption for sensitive environments
- See [safety-protocols.md](./architecture/safety-protocols.md) for details

## 📝 Contributing

1. Read the architecture documentation to understand system design
2. Follow the testing guidelines in [testing.md](./development/testing.md)
3. Use atomic commits with clear, thematic grouping
4. Run `task all:test` before submitting changes

## 📞 Support

For issues or questions:
- Check [TODO.md](../TODO.md) for known issues
- Review archived documentation for historical context
- Consult architecture docs for system understanding