# Analytical Access In mzdata

## Question

Can `mzdata` already support the kind of lazy loading and indexing needed for
analysis-oriented queries such as TIC, EIC, DT curves, and higher-dimensional
slice extraction?

Short answer: partially yes, but not completely.

The current system already has reader-level lazy access and some random-access
capabilities. That makes it good at loading spectra on demand and avoiding
eager materialization of every peak array. However, most of those capabilities
are still organized around the concept of a `Spectrum`, not around the concept
of an analytical query over RT, mobility, scan, and m/z dimensions.

## What exists today

### 1. Spectrum-oriented lazy loading

The `SpectrumLike` abstraction is centered on a single spectrum with accessors
such as `peaks()`, `raw_arrays()`, `ion_mobility()`, and summary updates. This
is a strong fit for spectrum inspection, identification workflows, and
format-neutral downstream code.

Relevant anchors:

- `src/spectrum/spectrum_types.rs:42`
- `src/spectrum/spectrum_types.rs:162`
- `src/spectrum/spectrum_types.rs:175`
- `src/spectrum/spectrum_types.rs:215`

This model helps avoid eagerly converting all source data into processed peaks,
but it still assumes that the spectrum is the main unit of work.

### 2. Reader-level random access

`mzdata` readers already support random access by spectrum index, ID, or time.
Combined with `DetailLevel::Lazy`, this lets a caller defer the expensive parts
of array access until needed.

This is useful, but it is still "give me spectrum N" lazy access, not "give me
only the subset of data needed to answer this analytical query".

### 3. A cross-format EIC query surface

The new `src/io/eic.rs` module introduces `EICQuery`,
`ExtractedIonChromatogram`, and `ExtractedIonChromatogramSource`. The default
implementation works by:

- normalizing query bounds
- switching the reader to `DetailLevel::Lazy`
- iterating spectra
- filtering spectra by RT/MS level/mobility
- summing only the matching m/z range in `raw_arrays()` or peak lists

Relevant anchors:

- `src/io/eic.rs:25`
- `src/io/eic.rs:128`
- `src/io/eic.rs:213`
- `src/io/eic.rs:244`

This is a meaningful improvement over full eager loading, but it is still a
query-over-spectra loop.

### 4. TDF-specific analytical fast path

Bruker TDF is currently the clearest example of analysis-oriented access in the
codebase. The TDF reader already has frame-level metadata, chromatogram
construction, and a dedicated `extract_eics_fast()` path that bypasses full
spectrum materialization.

Relevant anchors:

- `src/io/tdf/reader.rs:433`
- `src/io/tdf/reader.rs:491`
- `src/io/tdf/reader.rs:633`
- `src/io/tdf/reader.rs:672`

This implementation is much closer to what analytical extraction needs:

- map query bounds into native coordinates
- use frame/scan/TOF locality
- avoid building full `Spectrum` objects
- sum directly over a much smaller subset of data

## Why this is not yet enough

The gap is not that `mzdata` lacks lazy loading entirely. The gap is that its
lazy loading is mostly spectrum-centric.

For identification and general file compatibility, that is a reasonable default.
For analytical queries, it becomes limiting.

Examples:

- TIC extraction ideally wants precomputed or cheaply accumulated scan/frame
  totals, not full spectrum construction.
- DT curve extraction wants direct access along the mobility axis, not a loop
  over many spectrum objects that each expose only part of the mobility context.
- 2D slicing wants query-native access to a subset of the acquisition space,
  not repeated construction of intermediate spectrum containers.

In other words, the current system is good at:

- "load this spectrum lazily"
- "iterate spectra and inspect their arrays"

But analytical workloads want:

- "locate the relevant region first"
- "read only the local block"
- "aggregate without materializing unrelated spectra"

## Architectural conclusion

`Spectrum` should not be removed. It remains a useful compatibility and
inspection abstraction.

But it should stop being the only serious access path for analytical work.

The recommended direction is to keep the current spectrum-oriented model for:

- general reader interoperability
- identification workflows
- writing/exporting
- per-spectrum visualization and debugging

And add a second analysis-oriented layer for:

- EIC
- TIC
- DT curves
- mobility-filtered traces
- 2D or higher-dimensional slices

## Recommended model

### Layer 1: Spectrum layer

Keep the existing `SpectrumLike` / `MultiLayerSpectrum` path as the stable,
format-neutral object model.

This layer answers:

- what does a single spectrum look like?
- how do I inspect peaks, arrays, metadata, and precursor information?

### Layer 2: Index layer

Introduce a lighter-weight indexing abstraction that describes acquisition
coordinates and file-native locality, for example:

- RT range to frame/spectrum index mapping
- mobility range to scan range mapping
- m/z range to native coordinate hints where possible
- links to the underlying block or frame offsets

This layer should be cheap to build and cheap to cache.

### Layer 3: Trace / slice layer

Build analytical extraction APIs on top of the index layer instead of directly
on top of spectrum iteration.

Examples of future traits or entry points:

- `TraceSource`
- `AnalyticalIndex`
- `CubeSliceSource`
- `extract_tic`
- `extract_dt`
- `extract_eic_2d`

This layer should:

- accept query objects
- translate queries into local index lookups
- stream or accumulate only the relevant data blocks
- avoid constructing full `Spectrum` objects unless explicitly requested

## Practical recommendation for mzdata

Near term:

- keep the current `Spectrum` model
- keep the new cross-format EIC trait
- continue using TDF as the reference design for analysis-first extraction

Medium term:

- formalize a shared query/index abstraction
- separate "lazy spectrum access" from "lazy analytical access"
- add dedicated APIs for TIC and DT curves rather than forcing them through the
  spectrum pipeline

Long term:

- let each format implement the richest analytical access it can support
- keep a generic fallback for simpler formats
- preserve one public query API while allowing different backends to optimize
  differently

## Bottom line

`mzdata` already has enough lazy-loading machinery to support incremental
improvements, and it now has an initial cross-format EIC surface.

However, it does not yet have a complete, shared analytical indexing model that
fully matches high-dimensional extraction workloads.

So the answer is:

- yes, some of the building blocks already exist
- no, they are not yet fully aligned with the analytical use case
- the next real step is not replacing `Spectrum`, but adding an analytical
  access layer beside it
