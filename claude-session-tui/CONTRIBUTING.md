# Contributing to Claude Session TUI

Thank you for your interest in contributing to Claude Session TUI! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to conduct@anthropic.com.

## Ways to Contribute

### Report Bugs

Before creating bug reports, please check the issue list as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

* **Use a clear and descriptive title**
* **Describe the exact steps which reproduce the problem** in as many details as possible
* **Provide specific examples to demonstrate the steps**
* **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior
* **Explain which behavior you expected to see instead** and why
* **Include screenshots and animated GIFs if possible**
* **Include your operating system and Rust version**

### Suggest Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

* **Use a clear and descriptive title**
* **Provide a step-by-step description of the suggested enhancement** in as many details as possible
* **Provide specific examples to demonstrate the steps**
* **Describe the current behavior** and the suggested behavior
* **Explain why this enhancement would be useful**

### Pull Requests

* Follow the Rust style guide and code conventions
* Include appropriate tests for new functionality
* Update documentation as needed
* Ensure all tests pass locally before submitting
* Provide a clear description of the changes in the PR

## Development Setup

### Prerequisites

* Rust 1.70 or later (install from https://rustup.rs/)
* Cargo (comes with Rust)

### Local Development

```bash
# Clone the repository
git clone https://github.com/anthropics/claude-session-tui.git
cd claude-session-tui

# Build the project
cargo build

# Run tests
cargo test

# Run with TUI feature
cargo build --features tui
cargo run --features tui

# Run benchmarks
cargo bench
```

### Code Style

We follow standard Rust conventions:

* Use `cargo fmt` to format code
* Use `cargo clippy` to check for common mistakes
* Ensure all tests pass: `cargo test`
* Add documentation for public APIs

```bash
# Format code
cargo fmt

# Check for linting issues
cargo clippy -- -D warnings

# Run all checks before committing
cargo fmt && cargo clippy && cargo test
```

## Git Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes with clear messages
4. Push to your fork
5. Create a Pull Request with a description of your changes

### Commit Message Guidelines

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line
* Format: `type(scope): subject` (e.g., `feat(parser): add new file format support`)

Types: feat, fix, docs, style, refactor, perf, test, chore

## Testing

Please write tests for new functionality:

* Unit tests in the same file as the code being tested
* Integration tests in the `tests/` directory
* Benchmark tests for performance-critical code

Run tests with:

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

## Documentation

* Document public APIs with doc comments
* Update README.md for user-facing changes
* Add examples for complex features
* Keep documentation in sync with code changes

## Security

Please report security vulnerabilities to security@anthropic.com instead of using the issue tracker. See SECURITY.md for more details.

## Questions?

Feel free to open a GitHub issue with the `question` label or reach out to the maintainers.

## Recognition

Contributors will be recognized in the project's CONTRIBUTORS.md file (created as the project grows).

Thank you for contributing!
