---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
current_phase: 1
current_phase_name: Public EIC Surface
current_plan: 2
status: executing
stopped_at: Completed 01-02-PLAN.md
last_updated: "2026-03-25T06:23:14.537Z"
last_activity: 2026-03-25
progress:
  total_phases: 4
  completed_phases: 1
  total_plans: 2
  completed_plans: 2
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-03-25)

**Core value:** A caller can extract targeted analytical traces through one
clean API without loading the whole file or writing format-specific extraction
code.
**Current focus:** Phase 1 - Public EIC Surface

## Current Position

**Phase:** 1 of 4 (Public EIC Surface)
**Plan:** 2 of 2 in current phase
**Status:** Phase complete - ready for verification
**Last activity:** 2026-03-25 - Plan 01-02 executed for the public EIC export/discoverability pass.
**Progress:** [██████████] 100%

**Current Phase:** 1
**Current Phase Name:** Public EIC Surface
**Total Phases:** 4
**Current Plan:** 2
**Total Plans in Phase:** 2
**Last Activity:** 2026-03-25
**Last Activity Description:** Plan 01-02 finalized the public EIC export/discoverability pass,
mirrored the reader-native EIC surface across io/crate root/prelude, and added
discoverability cues.

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

## Session

**Last Date:** 2026-03-25T06:23:09.619Z
**Stopped At:** Completed 01-02-PLAN.md
surface discoverability.
**Resume File:** None
