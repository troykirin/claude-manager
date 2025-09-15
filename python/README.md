# Claude Manager TUI

Type-safe visual session management for Claude with Python 3.13+ features.

## Features

- 🎯 **Type-safe**: Full Python 3.13+ type safety with ReadOnly types
- 🚀 **Async**: High-performance async/await patterns for responsive UI
- 🎨 **Rich UI**: Beautiful terminal interface with Rich and prompt_toolkit
- 🔧 **Configurable**: Environment variable configuration support
- 🧪 **Tested**: Comprehensive test suite with benchmarks

## Installation

### Requirements

- Python 3.13+
- `cm` command available in PATH (see main project installation)

### Install from PyPI (when released)

```bash
pip install claude-manager-tui
```

### Development Installation

```bash
cd python/
pip install -e .[dev]
```

## Usage

### Console Script

After installation:

```bash
claude-manager-tui
```

### Direct Python Execution

```bash
# Simple TUI
python3 claude_manager_tui.py

# Type-safe TUI with async support
python3 claude_manager_tui_typed.py
```

## Configuration

Configure via environment variables:

```bash
export CM_COMMAND="claude-manager"          # Command to execute (default: cm)
export CLAUDE_MANAGER_BIN="/path/to/cm"     # Alternative command path
export CLAUDE_DIR="$HOME/.claude/projects"  # Claude directory
export CLAUDE_MAX_SESSIONS="20"             # Max concurrent sessions
export CLAUDE_SESSION_TIMEOUT="60.0"        # Session load timeout
```

## Development

### Setup Development Environment

```bash
python3 dev-setup.py
```

### Run Tests

```bash
pytest
pytest --benchmark-only  # Benchmarks only
```

### Type Checking

```bash
mypy claude_manager_tui_typed.py
pyright claude_manager_tui_typed.py
```

### Code Quality

```bash
ruff check .
black --check .
```

## Components

- **`claude_manager_tui.py`**: Simple TUI implementation
- **`claude_manager_tui_typed.py`**: Type-safe async implementation with Python 3.13+ features
- **`test_claude_manager_tui.py`**: Comprehensive test suite
- **`dev-setup.py`**: Development environment bootstrap

## License

MIT License - see main project for details.