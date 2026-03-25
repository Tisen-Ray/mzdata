---
phase: 01-public-eic-surface
plan: 01
subsystem: api
tags: [eic, mzreader, traits, tests]
requires:
  - phase: 01-public-eic-surface
    provides: Phase 1 context and the locked reader-native EIC decisions
provides:
  - Public EIC query/result contract docs in `src/io/eic.rs`
  - Reader-native dispatch commentary in `src/io/infer_format/dispatch.rs`
  - Focused public query-contract regression coverage in `src/io/infer_format/mod.rs`
affects: [01-02, phase-2]
tech-stack:
  added: []
  patterns: [reader-native trait entry points, additive public query/result surface, focused public API tests]
key-files:
  created: [".planning/phases/01-public-eic-surface/01-01-SUMMARY.md"]
  modified:
    - "src/io/eic.rs"
    - "src/io/infer_format/dispatch.rs"
    - "src/io/infer_format/mod.rs"
    - ".planning/STATE.md"
    - ".planning/ROADMAP.md"
key-decisions:
  - "Kept the Phase 1 public EIC surface centered on `reader.extract_eic(...)` and `reader.extract_eics(...)`."
  - "Documented the query object as a required `m/z` window with optional RT, MS level, mobility, and minimum-intensity filters."
  - "Kept extracted EICs as a dedicated computed result type instead of folding them into file-native chromatogram semantics."
patterns-established:
  - "Pattern 1: EIC builder methods remain chainable and `#[must_use]` to reinforce the public query contract."
  - "Pattern 2: Dispatch commentary stays with `MZReaderType` so the shared reader API remains the obvious caller surface."
requirements-completed: [EIC-01, EIC-02, ARCH-01]
duration: 50m
completed: 2026-03-25
---

# Phase 1: Public EIC Surface Summary

The reader-native Phase 1 EIC surface is now explicit in the code and pinned by
focused tests.

## Performance

- Duration: 50m
- Started: 2026-03-25T13:28:00+08:00
- Completed: 2026-03-25T14:17:03+08:00
- Tasks: 3
- Files modified: 6

## Accomplishments

- Documented the public `EICQuery`, `ExtractedIonChromatogram`, and
  `ExtractedIonChromatogramSource` contract in `src/io/eic.rs`.
- Kept `MZReaderType` dispatch aligned with the normal reader-style
  `extract_eic` / `extract_eics` calling path.
- Added public query-contract regression coverage for the full Phase 1 filter
  set and a result-construction sanity check.

## Task Commits

No atomic commits were created in this pass because the changes overlapped in
`src/io/eic.rs` and were kept together as one verified working-tree change set.

## Files Created/Modified

- `.planning/phases/01-public-eic-surface/01-01-SUMMARY.md` - plan summary
- `src/io/eic.rs` - contract docs, builder polish, and result/query tests
- `src/io/infer_format/dispatch.rs` - dispatch-side reader-native EIC comment
- `src/io/infer_format/mod.rs` - public query-contract regression test
- `.planning/STATE.md` - phase progress tick
- `.planning/ROADMAP.md` - phase progress tick

## Decisions Made

- The shared EIC API stays reader-native instead of becoming a separate
  service-style abstraction.
- The public query shape remains intentionally small for Phase 1.
- Extracted ion chromatograms stay as a computed result type distinct from
  file-native chromatograms.

## Deviations from Plan

None.

## Issues Encountered

- The default Windows cargo build path hit a `libz-sys` CMake generator error,
  so verification used `cargo test extract_eic --lib --no-default-features --features mzml,miniz_oxide,nalgebra` instead.
- That reduced-feature verification emitted one unrelated unused-import warning
  from `src/spectrum/scan_properties.rs`, outside the touched files.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 01-01 is complete and Phase 1 can move on to the export/discoverability
  pass in Plan 01-02.
- The public EIC contract is now documented and has a small regression anchor
  for the query builder surface.

---
*Phase: 01-public-eic-surface*
*Completed: 2026-03-25*
