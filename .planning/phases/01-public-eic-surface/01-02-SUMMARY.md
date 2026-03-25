---
phase: 01-public-eic-surface
plan: 02
subsystem: api
tags: [eic, exports, prelude, traits]
requires:
  - phase: 01-public-eic-surface
    provides: Phase 1 context and the locked reader-native EIC decisions
provides:
  - Public EIC exports on `mzdata::io`, crate root, and prelude
  - Discoverability cues that frame extracted EICs as computed reader views
  - Consistent prelude exposure for the reader-native EIC query/result/trait surface
affects: [phase-2, public-api]
tech-stack:
  added: []
  patterns: [flat additive re-exports, export-adjacent discoverability docs, reader-native EIC surface]
key-files:
  created:
    - ".planning/phases/01-public-eic-surface/01-02-SUMMARY.md"
  modified:
    - "src/io/mod.rs"
    - "src/lib.rs"
    - "src/prelude.rs"
    - ".planning/STATE.md"
    - ".planning/ROADMAP.md"
    - ".planning/REQUIREMENTS.md"
requirements-completed: [EIC-01, ARCH-01]
duration: 4m
completed: 2026-03-25
---

# Phase 1 Plan 02: Public EIC Surface Summary

**Flat reader-native EIC exports on `mzdata::io`, crate root, and prelude, with short docs that point callers to the computed reader-view API**

## Performance

- **Duration:** 4m
- **Started:** 2026-03-25T14:17:30+08:00
- **Completed:** 2026-03-25T14:21:10+08:00
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added export-adjacent doc cues in `src/io/mod.rs`, `src/lib.rs`, and `src/prelude.rs` so the public EIC surface is easy to find from the standard entry points.
- Kept the export layout flat and additive, with `src/io/eic.rs` still the focused implementation home.
- Aligned the prelude with the other public surfaces by re-exporting `EICError` alongside the EIC query, result, and trait types.

## Task Commits

No atomic commits were created for this workspace pass.

## Files Created/Modified

- `.planning/phases/01-public-eic-surface/01-02-SUMMARY.md` - plan summary
- `src/io/mod.rs` - io-level EIC export docs and re-export cue
- `src/lib.rs` - crate-root EIC export docs and re-export split
- `src/prelude.rs` - prelude EIC export alignment
- `.planning/STATE.md` - plan/state progression update
- `.planning/ROADMAP.md` - plan progress update
- `.planning/REQUIREMENTS.md` - requirement traceability update

## Decisions Made

- Kept the public EIC surface flat and mirrored across `io`, crate root, and prelude rather than adding any extra export layer.
- Added `EICError` to the prelude so the reader-facing EIC surface stays consistent across the common public entry points.

## Deviations from Plan

None - the execution stayed inside the export/discoverability scope.

## Issues Encountered

- `cargo test extract_eic --lib --no-default-features --features mzml,miniz_oxide,nalgebra` passed, but the build still emitted an unrelated unused-import warning from `src/spectrum/scan_properties.rs`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The public EIC surface is now easy to find from the normal `mzdata` export paths.
- Phase 2 can build on the same flat reader-oriented layout without revisiting the core EIC contract.

---
*Phase: 01-public-eic-surface*
*Completed: 2026-03-25*
