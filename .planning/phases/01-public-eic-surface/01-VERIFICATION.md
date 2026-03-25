---
phase: 01-public-eic-surface
status: passed
verified: 2026-03-25
requirements: [EIC-01, EIC-02, ARCH-01]
changed_files:
  - .planning/REQUIREMENTS.md
  - .planning/phases/01-public-eic-surface/01-VERIFICATION.md
---

# Phase 1 Verification: Public EIC Surface

## Result

Passed.

The Phase 1 goal is achieved: `mzdata` exposes a reader-native public EIC
surface through the normal reader hierarchy, the shared query type covers the
required Phase 1 filter set, and the API is exported from the standard public
entry points without introducing a separate service-style abstraction.

## Evidence

- `src/io/eic.rs` defines `EICQuery` with required `m/z` bounds and optional RT,
  MS level, mobility, and minimum-intensity filters.
- `src/io/eic.rs` defines `ExtractedIonChromatogramSource` with the reader
  entry points `extract_eic` and `extract_eics`.
- `src/io/infer_format/dispatch.rs` implements the shared EIC trait for
  `MZReaderType` and forwards through the existing reader dispatch surface.
- `src/io/mod.rs`, `src/lib.rs`, and `src/prelude.rs` re-export the EIC query,
  result, error, and trait types from the normal public surfaces.
- `src/io/eic.rs` and `src/io/infer_format/mod.rs` include focused tests for
  the reader dispatch path and the full Phase 1 query contract.

## Checks Run

```powershell
cargo test extract_eic --lib --no-default-features --features mzml,miniz_oxide,nalgebra
cargo test query_builder_preserves_the_full_phase_one_filter_set --lib --no-default-features --features mzml,miniz_oxide,nalgebra
cargo test test_eic_query_builder_carries_the_full_phase_one_filter_set --lib --no-default-features --features mzml,miniz_oxide,nalgebra
```

All three scoped checks passed.

## Gaps / Human Verification

- The default `cargo test extract_eic --lib` path is still blocked on this
  machine by the Windows `libz-sys` CMake generator setup, so I verified the
  phase with the scoped feature set used earlier in the milestone.
- I did not find a phase-code gap that changes the result: the implementation
  and focused tests already cover the Phase 1 goal.

## Notes

- `CLAUDE.md` is not present in this repository root.
- `git status` showed an unrelated modification in `.planning/config.json`; I
  left it untouched.
