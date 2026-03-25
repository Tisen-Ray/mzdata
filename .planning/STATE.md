---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
current_phase: 3
current_phase_name: additive backend integration
current_plan: Not started
status: planning
stopped_at: Completed 02-02-PLAN.md
last_updated: "2026-03-25T08:43:42.470Z"
last_activity: 2026-03-25
progress:
  total_phases: 4
  completed_phases: 2
  total_plans: 4
  completed_plans: 4
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-03-25)

**Core value:** A caller can extract targeted analytical traces through one
clean API without loading the whole file or writing format-specific extraction
code.
**Current focus:** Phase 3 - Additive Backend Integration

## Current Position

**Phase:** 3 of 4 (Additive Backend Integration)
**Plan:** Not started
**Status:** Ready to plan
**Last activity:** 2026-03-25
**Progress:** [██████████] 100%

**Current Phase:** 3
**Current Phase Name:** additive backend integration
**Total Phases:** 4
**Current Plan:** Not started
**Total Plans in Phase:** TBD
**Last Activity:** 2026-03-25
**Last Activity Description:** Phase 02 completed and the project transitioned
to Phase 3 for additive backend integration work.

## Accumulated Context

### Decisions

Decisions are logged in `.planning/PROJECT.md`. Current milestone assumptions:

- v1 keeps the shared public analytical model spectrum-oriented across formats.
- The portable strategy is per-spectrum lazy loading plus spectrum-internal
  binary search over ordered `m/z` arrays.
- TIC, DT, and broader analytical slicing stay deferred until the EIC surface
  is clean, documented, and validated.
- Phase 1 locks the public EIC entry point to reader trait methods
  (`extract_eic` / `extract_eics`) instead of helper/service-style access.
- Phase 1 keeps `ExtractedIonChromatogram` as a dedicated dynamic result type
  rather than reusing file-native chromatogram semantics.
- Phase 1 keeps the public query surface intentionally small: `m/z`, RT, MS
  level, mobility, and minimum intensity only.
- Phase 1 keeps `src/io/eic.rs` as a focused standalone module with normal
  re-exports.
- [Phase 1]: Kept the public EIC surface flat across io, crate root, and prelude — This preserves additive discoverability and avoids adding any extra export layer.
- [Phase 2]: Kept the portable fallback as a refinement of `extract_eics_from_spectra` with local lazy execution and ordered-array bounds rather than introducing a new abstraction layer.
- [Phase 02-portable-spectrum-eic-engine]: Kept the regression anchor synthetic and deterministic instead of adding a fixture-heavy harness. — This keeps the failure mode focused on chromatogram-shape drift and avoids brittle dataset dependencies.
- [Phase 02-portable-spectrum-eic-engine]: Reused the public reader-facing test neighborhood with a generic manual_extract helper rather than introducing a new abstraction layer. — This keeps the expected-result check close to the production extract_eic API while avoiding duplicated test scaffolding.

### Pending Todos

None yet.

### Blockers/Concerns

- The workspace is already dirty, so phase work should avoid broad formatting
  or unrelated cleanup.
- Real-data validation depends on access to
  `C:\Users\ray\Desktop\20241005-TDP-CYTO-C-1.raw`.
- Bruker TDF fast-path correctness and non-default feature coverage still need
  lightweight regression anchors before claiming cross-format confidence.

### Roadmap Evolution

- 2026-03-25: Initial v1 roadmap created with 4 phases focused on unified
  public EIC access.

## Performance Metrics

- 2026-03-25 plan 02-02: 9m, 3 tasks, 6 files modified

## Session

**Last Date:** 2026-03-25T08:43:42.470Z
**Stopped At:** Phase 02 complete, ready to plan Phase 03
**Resume File:** .planning/ROADMAP.md
