# Release Management Documentation - Review Draft

> **Status**: Draft for Review  
> **Scope**: Language-agnostic release processes for multi-component projects  
> **Target**: Storage in memory-kb for federation agent access

---

## 📋 RELEASE CHECKLIST

### Phase 1: Pre-Release Quality Gates

#### Code Quality & Standards
- [ ] **Linting**: All components pass language-specific linters
  - Bash: `shellcheck` with no warnings
  - Rust: `cargo clippy` with no warnings  
  - TypeScript: `eslint` with configured rules
  - Python: `ruff` or `black` + `isort`
- [ ] **Formatting**: Consistent code formatting applied
- [ ] **Static Analysis**: Security and quality scans completed
- [ ] **Dependency Audit**: No known vulnerabilities in dependencies
- [ ] **License Compatibility**: All dependencies have compatible licenses

#### Testing & Validation
- [ ] **Unit Tests**: All unit tests passing with ≥80% coverage
- [ ] **Integration Tests**: Cross-component integration verified
- [ ] **End-to-End Tests**: Full user workflows tested
- [ ] **Performance Tests**: No regression in key metrics
- [ ] **Compatibility Tests**: Tested on target platforms/versions
- [ ] **Manual Testing**: Critical user paths validated

#### Documentation & Communication
- [ ] **README Updates**: Installation, usage, and examples current
- [ ] **API Documentation**: Generated docs for public interfaces
- [ ] **CHANGELOG**: All changes documented with impact assessment
- [ ] **Breaking Changes**: Clearly documented with migration guides
- [ ] **Release Notes**: Draft prepared with user-facing changes
- [ ] **Internal Documentation**: Architecture and maintenance docs updated

#### Version Management
- [ ] **Version Consistency**: All components use coordinated versions
- [ ] **Semantic Versioning**: MAJOR.MINOR.PATCH properly applied
- [ ] **Version Constants**: Version embedded in code where needed
- [ ] **Dependency Versions**: Pinned or properly constrained
- [ ] **Backward Compatibility**: Breaking changes justified and documented

#### Security & Compliance
- [ ] **Security Review**: No exposed secrets or credentials
- [ ] **Vulnerability Scan**: No critical or high-severity issues
- [ ] **License File**: Current and comprehensive license information
- [ ] **Third-party Notices**: All required attributions included
- [ ] **Export Controls**: Compliance with applicable regulations

#### Multi-Component Coordination
- [ ] **Build Order**: Components build in correct dependency order
- [ ] **Interface Contracts**: APIs between components stable
- [ ] **Configuration**: Environment variables and configs documented
- [ ] **Installation**: End-to-end installation process verified
- [ ] **Cross-Component Tests**: Integration points validated

---

## 📖 RELEASE RUNBOOK

### Phase 1: Pre-Flight Preparation

#### Environment Setup
1. **Clean Workspace**: Ensure working directory is clean
   ```bash
   git status --porcelain  # Should be empty
   git pull origin main    # Latest changes
   ```

2. **Version Planning**: Determine release type
   - **PATCH**: Bug fixes, no breaking changes
   - **MINOR**: New features, backward compatible
   - **MAJOR**: Breaking changes or significant rewrites

3. **Component Coordination**: For multi-component projects
   - Identify which components changed
   - Plan version bump strategy (synchronized vs independent)
   - Review cross-component dependencies

#### Pre-Release Testing
1. **Automated Test Suite**
   ```bash
   # Language-specific examples
   task all:test          # Task runner
   npm run test:ci        # Node.js
   cargo test --release   # Rust
   pytest --cov          # Python
   ```

2. **Manual Verification**
   - Install from scratch on clean system
   - Test critical user workflows
   - Verify documentation accuracy

### Phase 2: Version and Documentation

#### Version Updates
1. **Update Version Constants**
   ```bash
   # Examples for different languages
   sed -i 's/VERSION=".*"/VERSION="X.Y.Z"/' script.sh
   cargo set-version X.Y.Z
   npm version X.Y.Z --no-git-tag-version
   ```

2. **Generate Changelog**
   ```bash
   # Using conventional commits
   conventional-changelog -p angular -i CHANGELOG.md -s
   # Or manual curation from git log
   git log --oneline $(git describe --tags --abbrev=0)..HEAD
   ```

3. **Update Documentation**
   - Version references in README
   - Installation instructions
   - API documentation regeneration

#### Quality Gates
1. **Final Test Run**
   ```bash
   # Comprehensive test with new versions
   task all:clean && task all:build && task all:test
   ```

2. **Security Final Check**
   ```bash
   # Scan for secrets
   git secrets --scan-history
   # Dependency audit
   npm audit --production
   cargo audit
   ```

### Phase 3: Release Execution

#### Git Operations
1. **Commit Release Changes**
   ```bash
   git add CHANGELOG.md package.json Cargo.toml
   git commit -m "chore(release): bump to vX.Y.Z"
   ```

2. **Create and Push Tag**
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin main --follow-tags
   ```

#### Build and Package
1. **Production Builds**
   ```bash
   # Rust
   cargo build --release
   # Node.js
   npm run build
   # Package creation
   tar -czf release-vX.Y.Z.tar.gz dist/
   ```

2. **Artifact Signing** (if required)
   ```bash
   gpg --armor --detach-sig release-vX.Y.Z.tar.gz
   sha256sum release-vX.Y.Z.tar.gz > release-vX.Y.Z.tar.gz.sha256
   ```

#### Distribution
1. **Package Registries**
   ```bash
   # npm
   npm publish
   # Cargo
   cargo publish
   # PyPI
   twine upload dist/*
   ```

2. **GitHub Release**
   ```bash
   gh release create vX.Y.Z \
     --title "Release vX.Y.Z" \
     --notes-file release-notes.md \
     release-vX.Y.Z.tar.gz
   ```

### Phase 4: Post-Release

#### Verification
1. **Installation Test**
   ```bash
   # Test installation from public sources
   npm install -g package-name@X.Y.Z
   cargo install crate-name --version X.Y.Z
   ```

2. **Smoke Tests**
   - Critical functionality working
   - No immediate user reports
   - Monitoring dashboards normal

#### Communication
1. **Announcements**
   - Release notes published
   - Team notifications sent
   - User community updated

2. **Documentation Updates**
   - Website/docs site updated
   - Wiki/knowledge base updated
   - Examples and tutorials refreshed

### Rollback Procedures

#### If Release Fails During Distribution
1. **Stop Distribution**
   ```bash
   # Unpublish if possible/appropriate
   npm unpublish package-name@X.Y.Z --force
   ```

2. **Revert Git Changes**
   ```bash
   git tag -d vX.Y.Z
   git push origin :refs/tags/vX.Y.Z
   git revert HEAD
   ```

#### If Critical Bug Discovered Post-Release
1. **Immediate Response**
   - Document the issue
   - Assess severity and impact
   - Communicate to users if needed

2. **Hotfix Process**
   - Create hotfix branch from release tag
   - Apply minimal fix
   - Fast-track through testing
   - Release as patch version

---

## 🎯 CLAUDE-MANAGER SPECIFIC RECOMMENDATIONS

### Current State Analysis
Based on analysis of `/Users/tryk/nabia/claude-manager`:

#### Missing Critical Elements
- ❌ No VERSION constant in `claude-manager.sh`
- ❌ No LICENSE file at repository root
- ❌ No CHANGELOG.md file
- ❌ No semantic versioning strategy
- ❌ No CI/CD pipeline
- ❌ No release automation

#### Existing Strengths
- ✅ Comprehensive Taskfile.yml with build automation
- ✅ Multi-component architecture well-structured
- ✅ Good documentation (README, technical docs)
- ✅ Working test suites for TUI and federation components
- ✅ Clean commit history with conventional commit style

### Recommended Immediate Actions

#### 1. Add Version Management
```bash
# Add to claude-manager.sh
VERSION="1.0.0"
```

#### 2. Create LICENSE File
```bash
# Choose appropriate license (MIT, Apache 2.0, etc.)
cp claude-session-tui/LICENSE ../LICENSE
```

#### 3. Initialize CHANGELOG
```bash
# Generate from git history
conventional-changelog -p angular -i CHANGELOG.md -s -r 0
```

#### 4. Add Version Sync Task
```yaml
# Add to Taskfile.yml
version:bump:
  desc: "Bump version across all components"
  vars:
    NEW_VERSION: '{{ .NEW_VERSION | default "patch" }}'
  cmds:
    - echo "Bumping version to {{.NEW_VERSION}}"
    # Update each component version
```

#### 5. Release Preparation Tasks
```yaml
# Add to Taskfile.yml
release:prepare:
  desc: "Prepare for release"
  cmds:
    - task: all:test
    - task: version:bump
    - conventional-changelog -p angular -i CHANGELOG.md -s

release:publish:
  desc: "Publish release"
  cmds:
    - git tag -a v{{.VERSION}} -m "Release v{{.VERSION}}"
    - git push origin main --follow-tags
    - gh release create v{{.VERSION}} --generate-notes
```

### Federation Integration Strategy

#### Memory-KB Storage Plan
1. **Entities to Create**:
   - `ReleaseProcess` - Main process entity
   - `QualityGate` - Individual checklist items
   - `ReleaseStep` - Runbook procedures
   - `LanguageSpecific` - Language-specific adaptations

2. **Relationships**:
   - `ReleaseProcess` → `contains` → `QualityGate`
   - `ReleaseProcess` → `implements` → `ReleaseStep`
   - `QualityGate` → `adapts_for` → `LanguageSpecific`

3. **Agent Access Patterns**:
   - Query by language: "rust release process"
   - Query by phase: "pre-release checklist"
   - Query by component: "multi-component versioning"

---

## 🚀 Next Steps

### For User Review
1. **Review Content**: Assess completeness and accuracy
2. **Customize for Project**: Identify project-specific adaptations
3. **Approve Storage**: Confirm memory-kb storage approach

### For Implementation
1. **Store in Memory-KB**: Create entities and relationships
2. **Fix Claude-Manager Gaps**: Implement missing elements
3. **Test Release Process**: Dry-run with patch release
4. **Document Lessons**: Update process based on experience

---

*This document serves as a comprehensive template for software release processes, designed to be language-agnostic while providing specific guidance for multi-component projects like claude-manager.*