# CONVENTIONS

## Scope
This document captures the conventions we follow when adding or refactoring rust code, keeping the `mzdata` crate aligned with its existing structure, formatting, and feature-gating practices.

## Module layout
The crate root (`src/lib.rs`) is the definitive re-export surface: it exposes `io`, `meta`, `params`, `prelude`, and `spectrum` modules, plus the most common helpers such as `MZReader`, `MzMLReader`, and `ExtractedIonChromatogram`. Keep new functionality nearby the closest domain module (`src/io` for format dispatch, `src/spectrum` for peak representations, etc.) and re-export it explicitly from `lib.rs` when it becomes part of the public API.

Within `src/io`, each format lives in its own submodule (e.g., `mzml`, `mgf`, `thermo`, `tdf`), and shared utilities are grouped under `infer_format`, `pipeline`, and `prelude`. Leverage the existing helpers (`MZReaderBuilder`, `DetailLevel`, `EICQuery`) rather than re-implementing low-level state handling, and guard platform-heavy code with the appropriate `#[cfg(feature = "...")]`.

## Rust style
- Favor idiomatic error handling with `Result`/`thiserror` and upstream types such as `mzpeaks::prelude`.
- Use `#[cfg(feature = "...")]` on modules, public re-exports, and tests whenever a dependency or functionality is gated (see `lib.rs` and `src/io/infer_format/mod.rs` for examples).
- Keep helpers private unless they need to be consumed by other modules; tests live in `#[cfg(test)] mod test` blocks adjacent to the code they exercise.
- Prefer descriptive doc comments (see the existing crate-level doc comments in `src/lib.rs`) and `mzdata`-style explicit re-export notes when you add new public helpers.

## Formatting
Run `cargo fmt --all` (Rust 2021 defaults) before submitting changes; there is no custom `rustfmt.toml`, so the formatter's default rules apply. The repository does not ship a separate formatting task in `Justfile`, but the standard Rust workflow keeps the codebase uniform.

## Feature-flag expectations
- The default feature set is `["zlib-ng-compat", "mgf", "mzml"]`. Anything touching mzML readers, writers, or `MZReader` fall under that umbrella, so they should compile with the defaults, but heavier formats such as `thermo`, `bruker_tdf`, `mzmlb`, or optional signal processing (`mzsignal`, `nalgebra`, `parallelism`) must remain behind `#[cfg(feature = "...")]`.
- When introducing new functionality that depends on an optional crate, add it to `Cargo.toml` under the right feature group, document the dependency in `src/lib.rs` (like the existing feature table), and mirror the gate in the code/tests so CI can opt into that feature via the `Justfile` tasks.
- Tests that rely on optional readers (e.g., `test_extract_eic_dispatch_tdf`) should carry the same `#[cfg(feature = "bruker_tdf")]` so that `cargo nextest run` only executes them when the feature is enabled, aligning with the `Justfile` test features: `nalgebra, parallelism, mzsignal, zlib-ng-compat, thermo, async, numpress, bruker_tdf`.

## Documenting CI expectations
The existing `Justfile` `test-units` target exercises all feature-heavy paths (`cargo nextest run --lib --features ...`). When you add new feature-gated code, ensure those features are mentioned in the `Justfile` target if the behaviour needs verification in the release test suite. Keep doc comments up to date so users can find the new APIs through `cargo doc`.
