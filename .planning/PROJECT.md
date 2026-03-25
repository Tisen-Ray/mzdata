# mzdata Analytical Access

## What This Is

This is a brownfield evolution of `mzdata` focused on analytical trace extraction
over large mass spectrometry datasets without forcing full-file materialization.
The immediate goal is to formalize a clean, public EIC access path that works
across formats while preserving `mzdata`'s existing reader, trait, and spectrum
architecture. The primary audience is `mzdata` maintainers and downstream users
who need cross-format extraction APIs without writing vendor-specific logic.

## Core Value

A caller can extract targeted analytical traces through one clean API without
loading the whole file or writing format-specific extraction code.

## Requirements

### Validated

- ✓ Unified multi-format reader dispatch already exists across mzML, MGF, Thermo
  RAW, mzMLb, imzML, and Bruker TDF readers.
- ✓ Spectrum access already supports index/ID/time lookup with configurable
  `DetailLevel` behavior and lazy spectrum-oriented loading where supported.
- ✓ Ion mobility and frame-oriented abstractions already exist for formats that
  expose richer acquisition structure, especially Bruker TDF.

### Active

- [ ] Formalize a public, shared EIC extraction interface for the library.
- [ ] Keep the trait layout, file structure, and reader layering elegant and
  easy to extend.
- [ ] Standardize on spectrum-level lazy access plus spectrum-internal binary
  search over ordered `m/z` arrays as the portable extraction model.
- [ ] Preserve compatibility with the existing spectrum/frame reader APIs while
  allowing format-specific optimizations behind the shared EIC surface.
- [ ] Add documentation and doc-tested examples that explain the new EIC access
  model and its limits.
- [ ] Add a small, practical validation path using
  `C:\Users\ray\Desktop\20241005-TDP-CYTO-C-1.raw`.

### Out of Scope

- Full unified TIC/DT/high-dimensional trace extraction in v1 — first make EIC
  clean, stable, and portable.
- A public cross-format API built around vendor-native frame/scan structures —
  the shared abstraction stays spectrum-oriented for now.
- Disk-level partial byte-range reads inside a single spectrum — v1 optimizes by
  reading spectra on demand, then performing in-memory binary search and local
  accumulation.
- A large benchmark suite over many real datasets — v1 only needs lightweight
  feasibility validation.

## Context

`mzdata` is an existing Rust crate with multi-format readers and a mature
`SpectrumLike`/`MultiLayerSpectrum` object model. Recent design work clarified
that the library should flatten format differences at the public API layer,
rather than exposing Bruker-specific frame/scan concepts as the shared
interface.

The key architectural decision is to keep `Spectrum` as the public common unit
of access while improving analytical extraction inside that model. That means:

- do not materialize the entire file up front
- read spectra on demand
- within each spectrum, use ordered `m/z` arrays and binary search to find the
  matching interval quickly
- allow richer format-specific backends, but keep the public contract uniform

There is already exploratory EIC work in the codebase, along with a TDF fast
path and codebase mapping documents under `.planning/codebase/`. This project
turns that direction into a cleaner, documented, and validated foundation.

## Constraints

- **Architecture**: Preserve the existing `mzdata` reader and trait model — the
  new EIC path must fit naturally into the current codebase.
- **Compatibility**: Avoid breaking the existing spectrum/frame/chromatogram
  APIs — analytical extraction should be additive.
- **Abstraction**: Keep the public API format-agnostic — vendor-native
  structures may optimize internally but should not dominate the shared
  interface.
- **Verification**: Provide documentation and a small real-data validation path
  — v1 must be understandable and demonstrably feasible.
- **Scope**: Finish EIC first — TIC, DT, and broader analytical slicing are
  intentionally deferred.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Keep public analytical extraction spectrum-oriented | This preserves cross-format uniformity and avoids leaking vendor-native acquisition models into the common API | — Pending |
| Use per-spectrum lazy loading plus in-memory `m/z` binary search as the common optimization strategy | This is portable across formats and significantly cheaper than full scans | — Pending |
| Treat EIC as the v1 analytical trace target | This keeps the first milestone narrow enough to stabilize design, docs, and tests before expanding to TIC/DT | — Pending |

---
*Last updated: 2026-03-25 after GSD project initialization*
