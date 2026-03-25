**Repository layout**
The root houses standard cargo metadata (`Cargo.toml`, `Cargo.lock`), the guide files (`README.md`, `CHANGELOG.md`, `Justfile`, `cliff.toml`), and several directories you should scan before coding: `docs/` for tutorials, `examples/` for runnable snippets, `benches/`/`test/` for regression suites, and `.planning/codebase/` for architecture work like this write-up.

**Source tree**
`src/lib.rs` stitches together the public API and re-exports the key crates (`io`, `spectrum`, `params`, `prelude`, `utils`, and optionally `mzsignal`). The `src/io` folder is the heavyweight layer for readers/writers, `src/spectrum` defines the data models (bindata, peaks, frame, chromatogram, group utilities), and `src/meta`, `src/params`, and `src/utils` host supporting metadata, parameter handling, and helpers seen across the project.

**IO submodules**
Inside `src/io`, pay attention to the gated format modules: `infer_format` (dispatch pipeline and `MZReader`), `eic.rs` (EIC traits), `tdf` (Bruker-specific reader + SQL helpers), `mzml`, `mzmlb`, `mgf`, `thermo`, `imzml`, `proxi`, and convenience helpers (`compression`, `shorthand`, `us i`). Feature flags turn these on or off for each build, so check `Cargo.toml` to align dependencies. `examples/` and `test/` both depend on these modules, so their use cases highlight common integration points.

**Documentation & helpers**
`docs/reader_tutorial.md`, `docs/writer_tutorial.md`, and `docs/spectrum_tutorial.md` are the best places for a newcomer to learn how mzdata expects readers, writers, and multi-layer spectra to behave before touching the north-star `spectrum` or `io` modules. The `src/tutorial.rs` module and the `docs/img/` assets can anchor visual patrols, while `.planning/codebase/` (this folder) consolidates architecture/structure return briefings so future contributors know where to drop new maps or supplements.
