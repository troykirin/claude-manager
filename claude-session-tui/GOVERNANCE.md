# Project Governance

## Overview

Claude Session TUI is an open-source project developed by Anthropic. This document describes the project governance structure and decision-making process.

## Project Leadership

### Maintainers

The project is maintained by the Anthropic team with oversight from the core maintainers group. Maintainers have:

* Merge authority on pull requests
* Authority to release new versions
* Authority to manage the project roadmap
* Responsibility for security and community health

### Code Owners

Specific components may have designated code owners with responsibility for review and maintenance. See CODEOWNERS file for details.

## Decision Making

### Proposals and RFCs

For significant changes, we use a Request for Comments (RFCs) process:

1. Open an issue or RFC for discussion
2. Community and maintainers review and discuss
3. Maintainers make final decision
4. Implementation follows approval

### Merge Policy

* All pull requests must be reviewed by at least one maintainer
* All tests must pass before merge
* Code style checks (clippy, fmt) must pass
* Documentation should be updated when necessary

### Release Process

1. Version bump in Cargo.toml following semver
2. Changelog updates
3. Release tag creation
4. Binary builds
5. Publication to crates.io

## Community Participation

### Ways to Participate

* Report bugs and request features via GitHub issues
* Submit pull requests with code improvements
* Improve documentation
* Help triage and discuss issues
* Provide feedback on proposals and RFCs
* Participate in the community

### Code of Conduct

All participants must adhere to our Code of Conduct (CODE_OF_CONDUCT.md). The community is expected to:

* Be respectful and inclusive
* Accept constructive criticism
* Focus on what is best for the community
* Show empathy toward others

## Contributor Recognition

We value and recognize contributions in several ways:

* Contributors are acknowledged in release notes
* Significant contributors may be added to CONTRIBUTORS.md
* Active community members may be invited to special discussions

## Changes to Governance

Changes to governance should be discussed openly and decided by the maintainers. Significant changes will be announced in the project README and community channels.

## Conflict Resolution

If conflicts arise:

1. **Direct Communication**: Attempt to resolve via direct, respectful communication
2. **Maintainer Review**: If unresolved, escalate to project maintainers
3. **Third-party Resolution**: If necessary, may involve additional parties
4. **Code of Conduct**: Violations are handled per Code of Conduct procedures

## Transparency

* Development happens in the open on GitHub
* Major decisions are documented in issues or RFCs
* Security reports are handled privately and disclosed responsibly
* Community is informed of significant changes

## License and IP

* All code is licensed under Apache 2.0
* Contributors grant license rights for their contributions
* Anthropic retains copyright on core infrastructure
* See LICENSE and NOTICE for full details

## Future Evolution

This governance document may evolve as the project grows. Changes will be:

* Discussed openly with the community
* Decided by core maintainers
* Announced clearly to all stakeholders
* Documented in the project history

## Contact

For governance questions or concerns, please:

* Open a GitHub issue with the `governance` label
* Email community@anthropic.com
* Contact a maintainer directly

## Acknowledgments

This governance structure is inspired by successful open-source projects and adapted for Claude Session TUI's needs. We are committed to maintaining a healthy, inclusive, and productive community.
