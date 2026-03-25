---
phase: 03-additive-backend-integration
plan: 01
subsystem: api
tags: [eic, tdf, fallback, dispatch, backend-routing]
requires:
  - phase: 02-portable-spectrum-eic-engine
    provides: Shared portable EIC baseline and regression anchors
provides:
  - TDF-native fast-path routing with an internal fallback gate
  - Portable fallback for partial mobility windows and other semantically risky query shapes
  - Dispatch comments that keep Bruker TDF optimization backend-local
affects: [phase-3-additive-backend-integration, EIC-05, ARCH-03]
tech-stack:
  added: []
  patterns:
    - backend-local fast-path selection
    - portable spectrum fallback seam
    - thin dispatch forwarding
key-files:
  created:
    - ".planning/phases/03-additive-backend-integration/03-01-SUMMARY.md"
  modified:
    - "src/io/eic.rs"
    - "src/io/tdf/reader.rs"
    - "src/io/infer_format/dispatch.rs"
    - ".planning/STATE.md"
    - ".planning/ROADMAP.md"
    - ".planning/REQUIREMENTS.md"
key-decisions:
  - "Kept the public EIC surface unchanged; TDF now decides internally whether a query batch can use the native fast path."
  - "Routed one-sided mobility windows to the shared portable engine so the native path stays semantically safe and easy to extend."
  - "Preserved dispatch as a thin forwarder, with the Bruker TDF branch staying backend-local instead of acquiring new public knobs."
patterns-established:
  - "Partial mobility bounds should fall back to the portable helper instead of being approximated by the TDF fast path."
  - "TDF routing checks belong in the backend reader, not in the format dispatch wrapper."
  - "Portable EIC regression coverage can anchor fallback semantics with synthetic spectra."
requirements-completed: [EIC-05, ARCH-03]
duration: 35m
completed: 2026-03-25
---

# Phase 3 Plan 01: Additive Backend Integration Summary

**Bruker TDF EIC extraction now chooses its native fast path internally and falls back to the shared portable engine when the query shape needs broader semantics**

## Performance

- **Duration:** 35m
- **Completed:** 2026-03-25T09:42:55Z

## Accomplishments

- Added a backend-local routing gate in `src/io/tdf/reader.rs` so Bruker TDF only uses its native fast path when the query batch has complete mobility windows.
- Kept the shared portable engine in `src/io/eic.rs` as the semantic fallback and added a regression for one-sided mobility filtering to prove the fallback seam stays usable.
- Left `src/io/infer_format/dispatch.rs` lightweight by preserving the shared trait forwarder and documenting that Bruker TDF keeps the optimization decision internally.

## Verification

- `cargo test --lib io::eic::tests::`
- `cmd /c '"C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64 >nul && set "CMAKE_GENERATOR=Ninja" && cargo test --lib io::tdf::reader::test::native_fast_path_requires_complete_mobility_windows --features bruker_tdf,nalgebra -- --exact'`
- `cmd /c '"C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=amd64 -host_arch=amd64 >nul && set "CMAKE_GENERATOR=Ninja" && cargo test --lib io::infer_format::test::test_extract_eic_dispatch_tdf --features bruker_tdf,nalgebra -- --exact'`

## Files Created/Modified

- `.planning/phases/03-additive-backend-integration/03-01-SUMMARY.md` - plan summary
- `src/io/eic.rs` - portable one-sided mobility regression
- `src/io/tdf/reader.rs` - backend-local fast-path-or-fallback routing
- `src/io/infer_format/dispatch.rs` - dispatch comment clarifying backend-local optimization
- `.planning/STATE.md` - plan/state progression update
- `.planning/ROADMAP.md` - phase progress update
- `.planning/REQUIREMENTS.md` - requirement traceability update

## Deviations from Plan

None. The implementation stayed inside the additive backend-integration scope.

## Issues Encountered

- `rustfmt` panicked on `src/io/infer_format/dispatch.rs` under the local toolchain, so formatting was left as-is after a manual review.
- The Bruker TDF verification needed the Visual Studio developer environment and `nalgebra` because `bruker_tdf` pulls in `mzsignal`'s linear-algebra backend.

## Next Phase Readiness

Phase 3 plan 01 is complete. Plan 03-02 can now focus on the remaining backend-integration regression coverage without revisiting the public EIC surface.

---
*Phase: 03-additive-backend-integration*
*Completed: 2026-03-25*
