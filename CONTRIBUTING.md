# Contributing to Claude Manager

Thank you for your interest in contributing to **claude-manager**!

## Getting Started

1. **Fork the repository**
   ```bash
   git clone https://github.com/yourusername/claude-manager.git
   cd claude-manager
   ```

2. **Install development dependencies**
   ```bash
   make check-deps
   ```

3. **Run tests**
   ```bash
   make test
   ```

## Development Workflow

### Making Changes

1. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes to `src/claude-manager.sh`

3. Test your changes:
   ```bash
   # Run tests
   bash tests/test_basic.sh

   # Test manually
   bash src/claude-manager.sh help
   ```

4. Update documentation if needed:
   - `README.md` - Overview and quick start
   - `docs/usage/USAGE.md` - Command reference
   - `docs/installation/INSTALLATION.md` - Installation guide

### Code Style

- Use Bash 4.4+ features
- Follow existing code structure
- Add comments for complex logic
- Use `_prefix` for internal functions
- Validate inputs and handle errors gracefully

### Testing

Before submitting:

```bash
# Run all tests
make test

# Check dependencies
make check-deps

# Test installation
./install.sh

# Manual testing
cm help
cm list
CLAUDE_DRY_RUN=true cm migrate "/test/old" "/test/new"
```

## Submitting Changes

1. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: Add descriptive commit message"
   ```

2. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

3. **Open a Pull Request**
   - Describe your changes
   - Reference any related issues
   - Include test results

## Reporting Issues

When reporting issues, include:
- Bash version: `bash --version`
- OS/Platform: macOS, Linux, WSL, etc.
- Python version: `python3 --version`
- Error messages and logs
- Steps to reproduce

## Feature Requests

We welcome feature requests! Please:
- Check existing issues first
- Describe the use case
- Explain expected behavior
- Consider backward compatibility

## Code of Conduct

- Be respectful and constructive
- Follow best practices for shell scripting
- Prioritize backward compatibility
- Test on multiple platforms when possible

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Open an issue or discussion on GitHub!
