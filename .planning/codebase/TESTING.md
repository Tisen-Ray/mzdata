# TESTING

## Test runners
- `just test-units` (alias `just t`) wraps `cargo nextest run --lib --features nalgebra,parallelism,mzsignal,zlib-ng-compat,thermo,async,numpress,bruker_tdf` and is the simplest way to exercise the default plus heavy optional features. It automatically loads `.env` because `Justfile` sets `dotenv-load := true`.
- `cargo nextest run` is the underlying command; pass `--features ...` yourself if you only need a subset or prefer a different feature matrix.
- `just test-coverage` executes `cargo llvm-cov` with the same feature bundle plus `mzmlb` and produces HTML coverage reports under `target/llvm-cov`.
- For quick iterations you can also run `cargo test --lib --features ... MZReader` or `cargo test infer_mzml` to target specific modules or test names.

## Feature gating
- Default cargo features already enable `mzml`, `mgf`, and `zlib-ng-compat`, so the core tests (e.g., `infer_mzml`, `test_extract_eic_dispatch_mzml`, `test_dispatch_mzreader`) work out of the box.
- Enable `thermo` to run the `infer_thermo`/`infer_open_thermo` tests, and enable `bruker_tdf` when exercising the TDF-specific behavior (`test_extract_eic_dispatch_tdf` and the ignored Bruker desktop checks). All `bruker_tdf` tests also rely on `mzsignal`/`mzsignal`’s feature set via the curated `Justfile` bundle.
- The `mzml` feature is also required for the EIC verification suite because the primary fixtures are under `test/data/*.mzML`.

## EIC regression coverage (src/io/infer_format/mod.rs)
- `manual_extract` and `peak_queries_from_first_ms1` are the trusted reference implementations used across the EIC tests. They iterate every spectrum exposed by `MZReader`, respect `DetailLevel`, filter by `EICQuery`, and sum intensities from either raw arrays or centroid peaks.
- `test_extract_eic_dispatch_mzml` compares the fast `reader.extract_eic(&query)` pathway against `manual_extract` for `test/data/small.mzML`. It validates that `times` and `intensities` have the same length, that each time coordinate matches within `1e-9`, and that intensity differences stay below `1e-3`.
- `test_extract_eics_dispatch_batch_mzml` repeats the comparison for `extract_eics` when multiple queries are batched, ensuring the fast batch extractor stays in lockstep with the manual sums.
- `test_extract_eic_dispatch_tdf` enables `#[cfg(feature = "bruker_tdf")]` and repeats the same manual-vs-fast comparison on the compressed Bruker TDF fixture at `test/data/diaPASEF.d`.

## Manual vs fast path verification tips
- The manual helper functions change the reader’s `DetailLevel` (lazy for extraction, full for peak selection) and reset it afterward, so tests can safely reuse the reader elsewhere.
- When evaluating new EIC logic, match the helper’s `mz` range filtering, `min_intensity` guard, and `rt_min/rt_max` checks to ensure the fast path doesn’t drop or duplicate points.
- Use the `manual_extract` function as a template: iterate every index, fetch a spectrum via `get_spectrum_by_index`, optionally read `raw_arrays()` or fall back to centroid peaks, and accumulate the intensity sum for each query.
- For batched queries, mirror the code in `test_extract_eics_dispatch_batch_mzml` to compare results pairwise and assert the same point counts before checking tolerances.

## Ignored Bruker desktop regression tests
- There are three additional `#[ignore = "..."]` tests in `src/io/infer_format/mod.rs` (`test_extract_eic_bruker_desktop_regression`, `test_extract_eic_bruker_desktop_smoke`, `test_extract_eic_bruker_desktop_fast_bench`). They exercise two paths:
  - Manual validation of `extract_eic` against `manual_extract` using three MS1 queries derived from real Bruker desktop datasets under `C:\Users\ray\Desktop\MS_Data_20260318`.
  - Performance benchmarking of the fast path only (fast/vs manual time prints).
- To run them manually, provide the datasets and use `cargo nextest run --lib --features nalgebra,parallelism,mzsignal,zlib-ng-compat,thermo,async,numpress,bruker_tdf -- --ignored`. Each test prints fast/manual elapsed times and per-query diagnostics, so pay attention to any `assert!` failure referencing `max_abs_diff` or missing files.
- Because the datasets are large, keep these tests ignored in CI; re-enable them temporarily when you can stage the raw data and want to double-check regression-sensitive logic.

## Supporting fixtures
- Core unit tests rely on files under `test/data/` (e.g., `small.mzML`, `small.mgf`, `small.RAW`, zipped mzML, `diaPASEF.d`, and synthetic `MGF`/`mzMLb` fixtures). Keep these files in place; if you need new fixtures, add them alongside the existing ones and reference them via `path::Path::new(...)`.
- `infer_from_stream` also validates gzipped streams (see `test_extract_eic_dispatch_mzml` for zipped detection via `test/data/20200204_...mzML.gz`).

## Verification checklist
1. Run `just test-units` to cover the full suite with the standard feature bundle.
2. Confirm `manual_extract` vs `extract_eic`/`extract_eics` comparisons for mzML remain within the tolerances listed above when you change the EIC implementation.
3. Enable `bruker_tdf` and rerun the TDF-specific test to make sure the fast path still matches manual sums.
4. If you need desktop-level regression proofs, fetch the Bruker dataset and run the ignored suite with `--ignored`, verifying the printed `max_abs_diff` remains below `1e-3`.
5. After touching formatting-sensitive code, run `cargo fmt --all` to avoid style churn.
