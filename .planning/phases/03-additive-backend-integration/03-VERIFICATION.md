---
phase: 03-additive-backend-integration
status: passed
verified: 2026-03-25
requirements: [EIC-05, ARCH-02, ARCH-03]
changed_files:
  - src/io/eic.rs
  - src/io/tdf/reader.rs
  - src/io/infer_format/dispatch.rs
  - src/io/infer_format/mod.rs
  - .planning/phases/03-additive-backend-integration/03-01-SUMMARY.md
  - .planning/phases/03-additive-backend-integration/03-02-SUMMARY.md
  - .planning/phases/03-additive-backend-integration/03-VERIFICATION.md
---

# Phase 3 Verification: Additive Backend Integration

## Result

Passed.

The Phase 3 goal is achieved: backend-specific acceleration remains hidden
behind the existing public EIC API, the Bruker TDF reader now chooses its fast
path internally with an explicit portable fallback for one-sided mobility
queries, and the reader-facing regression suite now proves both the optimized
and fallback paths while protecting existing spectrum and frame behavior.

## Evidence

- [src/io/tdf/reader.rs](D:/Projects/mzdata/src/io/tdf/reader.rs) keeps the
  optimization decision backend-local via `can_use_native_eic_fast_path` and
  falls back to the shared portable engine for partial mobility windows.
- [src/io/eic.rs](D:/Projects/mzdata/src/io/eic.rs) remains the semantic
  baseline and includes a portability regression for one-sided mobility
  filtering.
- [src/io/infer_format/dispatch.rs](D:/Projects/mzdata/src/io/infer_format/dispatch.rs)
  still forwards through the shared trait without adding a public backend
  selector.
- [src/io/infer_format/mod.rs](D:/Projects/mzdata/src/io/infer_format/mod.rs)
  adds reader-facing fast-path, fallback, and compatibility tests through the
  public `extract_eic` / `extract_eics` API.

## Checks Run

```powershell
cmd /c "call ""C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat"" -arch=amd64 -host_arch=amd64 >nul && set ""CMAKE_GENERATOR=Ninja"" && cargo test --lib io::infer_format::test:: --features bruker_tdf,nalgebra"
```

The targeted Phase 3 reader-facing suite passed:

- `test_extract_eic_dispatch_tdf_fast_path_matches_manual_reference`
- `test_extract_eic_dispatch_tdf_portable_fallback_matches_manual_reference`
- `test_tdf_spectrum_and_frame_compatibility_after_eic_integration`

Wave-level verification also passed earlier for:

- `io::eic::tests::`
- `io::tdf::reader::test::native_fast_path_requires_complete_mobility_windows`
- `io::tdf::reader::test::native_fast_path_matches_portable_reference_for_supported_query_shape`

## Gaps / Human Verification

- The ignored desktop Bruker regression tests remain optional machine-local
  checks and were not part of the default verification pass.
- Test output still includes an existing unused-import warning in
  `src/spectrum/scan_properties.rs`; this predates Phase 3 and does not affect
  the phase goal.

## Notes

- The fallback regression uses the portable EIC helper as the oracle for
  mobility-filtered synthetic coverage because the existing `manual_extract`
  helper does not model mobility filtering.
- I verified completion by file state and test results; this execution did not
  produce per-plan git commits.
