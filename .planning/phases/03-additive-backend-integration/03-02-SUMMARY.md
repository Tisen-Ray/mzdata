---
phase: 03-additive-backend-integration
plan: 02
subsystem: api
tags: [eic, tdf, fallback, compatibility, regression]
requires:
  - phase: 03-additive-backend-integration
    plan: 01
    provides: Backend-local TDF routing and portable fallback seam
provides:
  - Reader-facing fast-path regression coverage for TDF through the shared public EIC API
  - Portable fallback validation for one-sided mobility queries through the shared public EIC API
  - Explicit spectrum and frame compatibility checks around the EIC integration seam
affects: [phase-3-additive-backend-integration, EIC-05, ARCH-02]
tech-stack:
  added: []
  patterns:
    - public EIC regression coverage
    - portable reference comparison
    - spectrum/frame compatibility proof
key-files:
  created:
    - ".planning/phases/03-additive-backend-integration/03-02-SUMMARY.md"
  modified:
    - "src/io/infer_format/mod.rs"
    - "src/io/tdf/reader.rs"
    - ".planning/STATE.md"
    - ".planning/ROADMAP.md"
    - ".planning/REQUIREMENTS.md"
key-decisions:
  - "Kept the public EIC API as the only caller-visible entry point and exercised both the optimized and fallback TDF paths through that surface."
  - "Used the portable EIC helper as the oracle for fallback coverage, since the shared manual_extract regression does not model mobility filtering."
  - "Placed the spectrum/frame compatibility proof next to the public EIC regression tests so ARCH-02 stays visible in the same reader-facing neighborhood."
patterns-established:
  - "Fast-path queries should be checked against the manual spectrum oracle."
  - "Fallback queries that depend on mobility filtering should be checked against the portable EIC helper."
  - "Compatibility checks should cover spectrum lookup, iterator reset, and frame access without adding new public knobs."
requirements_completed: [ARCH-02]
duration: 35m
completed: 2026-03-25
---

# Phase 3 Plan 02: Additive Backend Integration Summary

**TDF EIC regression coverage now proves both the optimized path and the portable fallback path through the shared public API, while protecting existing spectrum and frame behavior**

## Performance

- **Duration:** 35m
- **Completed:** 2026-03-25T10:19:08Z

## Accomplishments

- Added a backend-local TDF regression in `src/io/tdf/reader.rs` that compares `extract_eics_fast` against the portable helper for a supported fast-path query.
- Split the reader-facing TDF coverage in `src/io/infer_format/mod.rs` into explicit fast-path and fallback tests, both exercised through the public `extract_eic` / `extract_eics` API.
- Added a compatibility check in `src/io/infer_format/mod.rs` that verifies spectrum lookup, iterator reset, and frame access still behave normally after EIC extraction.
- Reviewed the ignored desktop Bruker regressions and kept them aligned with the shared public API and manual-reference style.

## Verification

- `cmd /c '"C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64 >nul && set "CMAKE_GENERATOR=Ninja" && cargo test --lib io::tdf::reader::test::native_fast_path_matches_portable_reference_for_supported_query_shape --features bruker_tdf,nalgebra -- --exact'`
- `cmd /c '"C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64 >nul && set "CMAKE_GENERATOR=Ninja" && cargo test --lib io::infer_format::test::test_extract_eic_dispatch_tdf_fast_path_matches_manual_reference --features bruker_tdf,nalgebra -- --exact'`
- `cmd /c '"C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64 >nul && set "CMAKE_GENERATOR=Ninja" && cargo test --lib io::infer_format::test::test_extract_eic_dispatch_tdf_portable_fallback_matches_manual_reference --features bruker_tdf,nalgebra -- --exact'`
- `cmd /c '"C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64 >nul && set "CMAKE_GENERATOR=Ninja" && cargo test --lib io::infer_format::test::test_tdf_spectrum_and_frame_compatibility_after_eic_integration --features bruker_tdf,nalgebra -- --exact'`

## Files Created/Modified

- `.planning/phases/03-additive-backend-integration/03-02-SUMMARY.md` - plan summary
- `src/io/infer_format/mod.rs` - public reader-facing fast-path, fallback, and compatibility regressions
- `src/io/tdf/reader.rs` - backend-local fast-path comparison regression
- `.planning/STATE.md` - phase progress update
- `.planning/ROADMAP.md` - phase completion update
- `.planning/REQUIREMENTS.md` - ARCH-02 traceability update

## Deviations from Plan

- The fallback regression uses the portable EIC helper as the reference oracle instead of the synthetic `manual_extract` helper, because the synthetic helper does not model mobility filtering.

## Issues Encountered

- `cargo test` still emits an existing unused-import warning from `src/spectrum/scan_properties.rs`; it predates this phase and was not touched.

## Next Phase Readiness

Phase 3 is complete. The roadmap can now move to Phase 4 planning for docs and feasibility validation.

---
*Phase: 03-additive-backend-integration*
*Completed: 2026-03-25*
