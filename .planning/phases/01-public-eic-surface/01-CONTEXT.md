# Phase 1: Public EIC Surface - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase defines the shared public EIC surface for `mzdata`: the public query
type, result type, trait entry points, and module/export layout that
cross-format callers will use. It does not expand scope into TIC, DT, broader
analytical slicing, or deeper performance work beyond what is needed to make
the public surface coherent.

</domain>

<decisions>
## Implementation Decisions

### Public entry point
- The public EIC interface stays as a reader trait extension.
- The intended calling style is `reader.extract_eic(...)` and
  `reader.extract_eics(...)`.
- The trait continues to sit naturally beside the existing reader APIs and
  dispatch through `MZReader`.

### Result semantics
- The public result type remains a dedicated
  `ExtractedIonChromatogram`.
- Dynamic extraction results must not be conflated with file-native
  `Chromatogram` objects.
- No automatic reuse of the `Chromatogram` return surface is required in this
  phase.

### Query scope
- Phase 1 formally supports only the current shared query fields:
  `m/z` range, optional RT range, optional MS level, optional mobility range,
  and optional minimum intensity.
- More advanced filtering semantics, tolerance modes, polarity, precursor-window
  filtering, or custom aggregation behavior are explicitly deferred.
- Higher-level orchestration of more complex filtering belongs above this layer.

### Module and export layout
- `src/io/eic.rs` remains a dedicated module.
- The EIC types and trait continue to be re-exported from `src/io/mod.rs` and
  the top-level library exports.
- The code should not be split into a deeper submodule tree for this phase.

### Claude's Discretion
- Exact method docs and naming polish around the existing public types.
- Minor internal helper extraction needed to keep `src/io/eic.rs` readable.
- Whether any tiny cleanup is needed in re-exports to make the public surface
  easier to discover.

</decisions>

<specifics>
## Specific Ideas

- Keep the public model consistent with the existing `mzdata` reader style
  rather than introducing a service-style API.
- Keep the result semantics explicit: extracted EICs are computed views, not
  native chromatograms from the underlying file.
- Keep the query object intentionally small because upper layers can compose
  richer behavior without making the base API harder to stabilize.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/io/eic.rs`: already contains `EICQuery`,
  `ExtractedIonChromatogram`, `ExtractedIonChromatogramSource`, and the shared
  cross-format extraction helper.
- `src/io/infer_format/dispatch.rs`: already wires the trait into `MZReaderType`
  so dispatch is a natural home for the public API surface.
- `src/io/mod.rs`, `src/lib.rs`, and `src/prelude.rs`: already export the new
  EIC types, which gives a clean additive path for finalizing the public API.

### Established Patterns
- Reader capabilities in `mzdata` are usually exposed as traits implemented by
  concrete readers and then forwarded through `MZReaderType`.
- `io` modules are kept relatively flat, with focused files and re-exports from
  `src/io/mod.rs`.
- Public interfaces are additive and format-agnostic, while richer
  format-specific behavior is hidden behind the same common surface.

### Integration Points
- The phase will primarily touch `src/io/eic.rs`, `src/io/mod.rs`,
  `src/io/infer_format/dispatch.rs`, and top-level exports in `src/lib.rs` /
  `src/prelude.rs`.
- The public surface must stay compatible with the existing `SpectrumSource`
  and `MZReader` usage model so later phases can add behavior without moving
  the API again.

</code_context>

<deferred>
## Deferred Ideas

- Public TIC extraction API
- Public DT extraction API
- Broader analytical slice APIs
- More expressive query semantics such as polarity, tolerance modes, or
  precursor-window filtering
- Deeper module splits under `src/io/eic/`

</deferred>

---
*Phase: 01-public-eic-surface*
*Context gathered: 2026-03-25*
