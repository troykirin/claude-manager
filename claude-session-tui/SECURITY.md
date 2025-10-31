# Security Policy

## Reporting a Vulnerability

The Claude Session TUI team and community take security bugs in Claude Session TUI seriously. We appreciate your efforts to responsibly disclose your findings.

Please do **NOT** report security vulnerabilities through public GitHub issues, discussions, or pull requests.

Instead, please report security vulnerabilities by emailing security@anthropic.com with:

* A description of the vulnerability
* Steps to reproduce the vulnerability
* Potential impact
* Your name and contact information (optional, but appreciated)

Please allow up to 90 days for us to respond and address the vulnerability.

## Security Considerations

### Session Files

Claude Session TUI processes session files that may contain:
* Conversation history
* User prompts and Claude responses
* File paths and system information
* Potentially sensitive context

**Best Practices:**
* Only process session files from trusted sources
* Be cautious when sharing session files with others
* Clear sensitive data from sessions before archiving
* Use appropriate file permissions on session directories

### File Operations

The tool performs file system operations including:
* Reading JSONL session files
* Creating temporary files during parsing
* Walking directory trees

**Best Practices:**
* Run from a trusted working directory
* Verify file permissions before processing
* Use absolute paths when possible
* Be cautious with symbolic links

### Data Privacy

Session files are processed locally on your machine:
* No data is sent to remote servers
* All processing is done in memory or temporary files
* Temporary files should be cleared appropriately

## Updates

Keep Claude Session TUI updated to receive security patches:

```bash
# Update via cargo
cargo install claude-session-tui --upgrade

# Or build from source
git clone https://github.com/anthropics/claude-session-tui.git
cd claude-session-tui
git pull origin main
cargo install --path .
```

## Supported Versions

Security updates are provided for the latest minor version. We recommend always using the latest stable release.

## Responsible Disclosure Timeline

We follow a 90-day responsible disclosure timeline:

1. **Day 0**: Vulnerability reported
2. **Day 1-7**: Initial assessment and confirmation
3. **Day 8-60**: Development and testing of fix
4. **Day 61-90**: Public disclosure and release
5. **Day 91**: Public announcement of vulnerability

## Security Audit

Claude Session TUI undergoes regular security reviews:

* Code review for security issues
* Dependency audits using `cargo audit`
* Testing on multiple platforms
* Community security reporting

## Dependencies

We carefully manage dependencies to minimize security risk:

* All dependencies are reviewed before inclusion
* Regular security audits with `cargo audit`
* Prompt updates to patch security vulnerabilities
* Minimize the number of dependencies where possible

## Questions

For security questions or concerns, please email security@anthropic.com.

## Changelog

Security-related changes are documented in the CHANGELOG with appropriate notification of any security implications.
