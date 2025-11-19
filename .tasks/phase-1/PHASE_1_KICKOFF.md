# Phase 1 Kickoff: Session Recovery Core
**Status**: Ready for Implementation | **Timeline**: 2 weeks | **Start**: 2025-10-28

---

## Sprint Overview

### Week 1: Governance Documentation
Focus: Define specification, runbook, architecture before any code

**Deliverables**:
- [ ] SESSION_RECOVERY_SPECIFICATION.md ✅ DONE
- [ ] SESSION_RECOVERY_RUNBOOK.md ✅ DONE
- [ ] SESSION_RECOVERY_ARCHITECTURE.md ✅ DONE
- [ ] Phase 1 test plan with validation gates
- [ ] CTech governance review + approval
- [ ] Linear board setup with Phase 1 tasks

**Owner**: Documentation Lead
**Exit Criteria**: All docs merged, CTech approval documented

### Week 2: Core Implementation
Focus: Health diagnostics + REPAIR mode + governance integration

**Deliverables**:
- [ ] Health diagnostic engine (`cm diagnose`)
- [ ] REPAIR mode recovery (`cm recover`)
- [ ] Recovery metadata schema + storage
- [ ] Integration with backup system (existing)
- [ ] Undo integration (`cm undo`)
- [ ] Dry-run support (`CLAUDE_DRY_RUN=true`)
- [ ] Audit log setup
- [ ] Unit tests (core paths)
- [ ] Integration tests (recovery workflow)

**Owner**: Implementation Team
**Exit Criteria**: Can recover corrupted session end-to-end with audit trail

---

## Directory Structure

```
~/.nabia/tools/claude-manager/
├── phase-1/                          # Phase 1 working directory
│   ├── PHASE_1_KICKOFF.md           # This file
│   ├── spec/
│   │   └── SESSION_RECOVERY_SPEC.md # Reference copy
│   ├── design/
│   │   ├── health_algorithm.md      # Health scoring design
│   │   ├── recovery_workflow.md      # Recovery state machine
│   │   └── governance_model.md       # Justification + audit
│   ├── tasks/
│   │   ├── week-1-docs.md
│   │   ├── week-2-implementation.md
│   │   └── validation-gates.md
│   └── tests/
│       ├── unit/
│       │   ├── test_health_scoring.py
│       │   ├── test_recovery_metadata.py
│       │   └── test_audit_trail.py
│       └── integration/
│           ├── test_recovery_workflow.py
│           └── test_undo_capability.py

~/Sync/docs/governance/
├── SESSION_RECOVERY_SPECIFICATION.md   ✅ DONE
├── SESSION_RECOVERY_RUNBOOK.md        ✅ DONE
└── SESSION_RECOVERY_AUDIT_FINDINGS.md (from Align)

~/Sync/docs/architecture/
├── SESSION_RECOVERY_ARCHITECTURE.md   ✅ DONE
└── SESSION_RECOVERY_ADR_001.md        (same as above)
```

---

## Week 1 Breakdown

### Monday-Tuesday: Documentation Review & Governance
- [ ] Review governance docs with CTech
- [ ] Address feedback (expected: minimal, high-level strategic alignment)
- [ ] Get written approval from CTech
- [ ] Schedule Linear board setup
- [ ] Create Phase 1 Linear project/issues

### Wednesday-Thursday: Test Plan & Design
- [ ] Write Phase 1 test plan
- [ ] Design health scoring algorithm (detailed)
- [ ] Design recovery state machine (workflow diagram)
- [ ] Design governance model (enum validation, audit fields)
- [ ] Design storage schema for recovery metadata

### Friday: Validation Gates & Setup
- [ ] Finalize validation gates (6 required tests)
- [ ] Set up Linear board with Phase 2-6 roadmap
- [ ] Create test file templates
- [ ] Create implementation templates
- [ ] Brief implementation team

---

## Week 2 Breakdown

### Monday-Tuesday: Health Diagnostics
- [ ] Implement health scoring algorithm
- [ ] Add JSONL parsing + error detection
- [ ] Add rendering risk assessment
- [ ] Add health trends tracking
- [ ] Unit tests for health engine
- [ ] Manual testing with corrupted sessions

### Wednesday: Recovery Core
- [ ] Implement REPAIR mode duplication
- [ ] Implement recovery metadata creation
- [ ] Implement backup integration
- [ ] Implement undo file creation
- [ ] Unit tests for recovery logic

### Thursday: Governance Integration
- [ ] Implement audit log
- [ ] Add justification prompts (enum validation)
- [ ] Add compliance tag support
- [ ] Add recovery chain tracking
- [ ] Implement duplication loop prevention

### Friday: Integration & Testing
- [ ] Integration tests (full recovery workflow)
- [ ] End-to-end testing (from corruption to recovered session)
- [ ] Manual testing with real corrupted sessions
- [ ] Undo testing (rollback scenarios)
- [ ] Documentation review + fixes

---

## Success Criteria (Phase 1 Exit)

### Functional
✅ `cm diagnose` identifies corrupted sessions with health scores
✅ `cm recover` recovers corrupted session with fresh UUID
✅ `cm recover-status` shows recovery history with audit trail
✅ `cm undo` rolls back recovery completely
✅ Backup created before every operation (verified)
✅ Undo file created (verified)

### Governance
✅ Audit log complete with all required fields
✅ Recovery reasons validated (enum only)
✅ Compliance tags supported (optional)
✅ Recovery chain tracked (generation + parent UUID)
✅ Duplication loop prevention tested (max depth = 2)

### Federation
✅ Loki event emission implemented (minimal Phase 1: recovery_initiated + recovery_completed)
✅ SurrealDB integration planned (Phase 2)
✅ memchain integration planned (Phase 2)

### Testing
✅ Unit tests for health scoring (>80% coverage)
✅ Unit tests for recovery metadata (>80% coverage)
✅ Unit tests for audit trail (>80% coverage)
✅ Integration tests for full workflow (critical path)
✅ All validation gates passed (6/6)

### Documentation
✅ Phase 1 docs merged to governance/architecture
✅ CTech approval documented
✅ Team onboarding materials ready
✅ Operational runbook updated
✅ Architecture decision recorded

---

## Linear Board Setup

### Epic: Session Recovery (v1.1)
- **Status**: In Progress
- **Phase**: 1/6
- **Owner**: Implementation Team
- **Timeline**: 6 weeks total (2 weeks per Phase 1-3, 1 week per Phase 4-6)

### Phase 1 Issues (Week 1-2)

```
Session Recovery Phase 1: Core Engine
├── CM-301: Health Diagnostics Engine
│   ├── CM-310: Implement health scoring algorithm
│   ├── CM-311: Add JSONL validation
│   ├── CM-312: Add rendering risk detection
│   └── CM-313: Unit tests + manual testing
├── CM-302: REPAIR Mode Recovery
│   ├── CM-320: Implement session duplication
│   ├── CM-321: Implement recovery metadata
│   ├── CM-322: Implement backup integration
│   └── CM-323: Unit tests + integration tests
├── CM-303: Governance Integration
│   ├── CM-330: Implement audit log
│   ├── CM-331: Add justification prompts
│   ├── CM-332: Add recovery chain tracking
│   └── CM-333: Implement duplication loop prevention
└── CM-304: Integration & Release
    ├── CM-340: End-to-end testing
    ├── CM-341: Undo capability testing
    ├── CM-342: Documentation review
    └── CM-343: CTech approval + merge
```

---

## Validation Gates

Before Phase 1 release, **ALL 6 gates must pass**:

1. **Health Scoring Gate**
   - [ ] Correctly identifies oversized content (>10,000 items)
   - [ ] Correctly scores corrupted sessions (<20 health)
   - [ ] Test case: your 19,695-item session → score must be <20

2. **Recovery Safety Gate**
   - [ ] Backup created before operation
   - [ ] Backup verified before proceeding
   - [ ] Undo capability tested (recovery completely reverted)

3. **Audit Trail Gate**
   - [ ] Every operation logged with timestamp
   - [ ] User/machine/reason recorded
   - [ ] Recovery ID generated and tracked
   - [ ] Immutable append-only format

4. **Chain Tracking Gate**
   - [ ] Recovery generation incremented correctly
   - [ ] Parent recovery ID stored
   - [ ] Duplication loop prevention (max depth = 2)
   - [ ] Orphan detection implemented

5. **Governance Gate**
   - [ ] CTech reviews specification
   - [ ] CTech approves audit model
   - [ ] Compliance tags supported (optional)
   - [ ] Written approval on file

6. **Federation Gate**
   - [ ] Loki events emitted (recovery_initiated, recovery_completed)
   - [ ] Events queryable via LogQL
   - [ ] Event schema matches specification
   - [ ] No event delivery failures

---

## Release Checklist

Before merge to main:

- [ ] All validation gates passed (6/6)
- [ ] Code review completed (2+ reviewers)
- [ ] Unit tests >80% coverage
- [ ] Integration tests pass
- [ ] Documentation complete and merged
- [ ] CTech approval documented
- [ ] Linear issues updated (all closed)
- [ ] Release notes drafted
- [ ] Version bump (v1.1.0)
- [ ] Tag created (release-v1.1.0)

---

## Team Roles

| Role | Responsibility | Hours/Week |
|------|-----------------|-----------|
| **Documentation Lead** | Spec, runbook, architecture reviews | 10h |
| **Implementation Lead** | Health engine, recovery core, governance | 20h |
| **Test Lead** | Test planning, validation gates, coverage | 15h |
| **Integration Lead** | Backup integration, undo, federation stubs | 10h |
| **CTech Reviewer** | Governance approval, compliance validation | 5h |

**Total Effort**: ~60 hours (1.5 FTE weeks)

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Underestimating complexity | Conservative timeline (2 weeks for 50h work) |
| Governance approval delays | Pre-approved architecture + early CTech engagement |
| Test coverage gaps | Defined validation gates + >80% code coverage target |
| Federation integration issues | Stubs prepared for Phase 2 |
| Documentation debt | Concurrent documentation with implementation |

---

## Next Actions (Today)

1. **Approve Phase 1 kickoff** ← User decision point
2. **Create Linear project** with Phase 1 issues
3. **Schedule CTech review** (tomorrow)
4. **Brief implementation team** (tomorrow)
5. **Kick off Week 1** (Day 1: Documentation Review)

---

## References

- **Specification**: ~/Sync/docs/governance/SESSION_RECOVERY_SPECIFICATION.md
- **Runbook**: ~/Sync/docs/governance/SESSION_RECOVERY_RUNBOOK.md
- **Architecture**: ~/Sync/docs/architecture/SESSION_RECOVERY_ARCHITECTURE.md
- **Full Architecture Vision**: Igris Strategic Analysis (Phase 1 docs)
- **Governance Audit**: Align Coherence Validation (Phase 1 docs)

---

**Status**: READY FOR IMPLEMENTATION
**Approval**: Pending CTech review (scheduled tomorrow)
**Timeline**: Weeks 1-2, start 2025-10-28
**Owner**: Implementation Team
