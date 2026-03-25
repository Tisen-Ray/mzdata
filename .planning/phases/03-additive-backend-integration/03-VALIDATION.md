---
phase: 03
slug: additive-backend-integration
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-25
---

# Phase 03 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib io::infer_format::test::test_extract_eic_dispatch_tdf --features bruker_tdf -- --exact` |
| **Full suite command** | `cargo test --lib io::infer_format::test:: --features bruker_tdf` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib io::infer_format::test::test_extract_eic_dispatch_tdf --features bruker_tdf -- --exact`
- **After every plan wave:** Run `cargo test --lib io::infer_format::test:: --features bruker_tdf`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | EIC-05 | integration | `cargo test --lib io::infer_format::test::test_extract_eic_dispatch_tdf --features bruker_tdf -- --exact` | existing | pending |
| 03-01-02 | 01 | 1 | ARCH-03 | unit | `cargo test --lib io::tdf::reader --features bruker_tdf extract_eic -- --nocapture` | existing | pending |
| 03-02-01 | 02 | 2 | ARCH-02 | integration | `cargo test --lib io::infer_format::test:: --features bruker_tdf` | existing | pending |
| 03-02-02 | 02 | 2 | EIC-05 | regression | `cargo test --lib io::infer_format::test::test_extract_eic_dispatch_tdf --features bruker_tdf -- --exact` | existing | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Bruker desktop ignored regression coverage on local datasets | EIC-05 | The desktop datasets are machine-local and not suitable for CI | Run the ignored Bruker desktop regression tests only when the local `.d` datasets are available and compare fast-path output against manual extraction. |

---

## Validation Sign-Off

- [x] All tasks have automated verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
