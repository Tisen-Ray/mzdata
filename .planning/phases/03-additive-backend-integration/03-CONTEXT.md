# Phase 3: Additive Backend Integration - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase keeps the public EIC surface introduced in phases 1-2 unchanged
while integrating backend-specific acceleration behind that same API. The work
must preserve existing spectrum and frame behavior, keep the portable shared
path intact as the baseline implementation, and leave a clean additive seam for
future TIC/DT work. It does not broaden scope into new analytical features,
public API redesign, or phase-4 documentation/real-data validation work.

</domain>

<decisions>
## Implementation Decisions

### Backend routing policy
- Keep the public entry point fixed at the existing reader-native
  `extract_eic` / `extract_eics` API. No new public knobs, backend selectors,
  or format-specific extraction methods are introduced in this phase.
- Treat the portable spectrum-based engine in `src/io/eic.rs` as the semantic
  baseline for all backends. Backend accelerations are internal optimizations
  that must preserve the same query/result contract.
- Keep the Bruker TDF fast path as the preferred route for supported queries
  because it is the main value of this phase, but do not make it an all-or-
  nothing dependency. Query shapes the fast path cannot satisfy confidently
  should fall back to the portable spectrum loop instead of forcing bespoke
  behavior.
- The integration should be framed as "optimized implementation behind the same
  trait" rather than "special public mode for one backend".

### Compatibility bar
- Existing spectrum access behavior must remain stable: `get_spectrum_by_*`,
  iteration, `DetailLevel`, and current flattened-spectrum behavior should keep
  working as before after EIC integration changes.
- Existing frame-oriented behavior for TDF must also remain stable. Phase 3 may
  reuse frame metadata and frame-local indexing internally, but it must not
  reshape the public frame reader model to serve EIC.
- EIC integration must stay additive. The planner should avoid refactors that
  move or rename existing public reader/frame APIs unless required for a narrow
  bug fix.
- If a backend optimization requires different internal data access patterns,
  adapt the implementation behind the trait rather than changing caller-facing
  semantics.

### Internal extension seam
- Keep `ExtractedIonChromatogramSource` as the single public capability trait.
- Introduce or clarify an internal seam for backend-specific query execution so
  that optimized backends can override the portable path without scattering EIC
  logic across unrelated modules.
- Prefer a structure where the portable engine remains reusable as an explicit
  fallback and backend readers opt into acceleration in their own module.
- The internal seam should be narrow and EIC-specific for now. Do not broaden
  this phase into a full shared analytical index layer; that larger design
  belongs to later TIC/DT-oriented work.

### Query and result semantics
- Backend accelerations must preserve the phase-1 and phase-2 query semantics:
  `m/z`, RT, MS level, mobility, and minimum-intensity filtering must mean the
  same thing regardless of backend.
- Result shape should continue to mirror the portable contract: when a spectrum
  or backend-native entry passes the spectrum-level filters, the EIC includes a
  time/intensity point even if the summed intensity is `0`.
- Mobility-aware backends may use native coordinates internally, but they must
  map those coordinates back to the same public mobility semantics already
  exposed by `EICQuery`.

### Regression coverage
- Add backend-integration tests that compare optimized output to the portable or
  manual reference path through the same public API surface.
- Phase 3 tests should explicitly protect `ARCH-02`: existing non-EIC spectrum
  and frame access still behaves as before after integration work.
- Add at least one focused regression check around the TDF fast path / fallback
  boundary so maintainers can see when an optimization silently diverges from
  portable semantics.
- Prefer lightweight, feature-gated checks in the existing reader-facing test
  neighborhood over heavy new harnesses.

### Claude's Discretion
- Exact internal helper names and module-local trait/function extraction used to
  represent the backend optimization seam.
- Whether the fallback decision is encoded as a query-capability predicate, a
  backend-local branch, or another small internal abstraction.
- Exact split of tests between `src/io/eic.rs`, `src/io/infer_format/mod.rs`,
  and backend-specific test modules.

</decisions>

<specifics>
## Specific Ideas

- No additional user requirements were supplied beyond "pick the best
  trade-off", so the preferred direction is conservative and additive:
  preserve the public surface, preserve compatibility, and hide acceleration
  behind internal backend-local execution.
- Treat the current Bruker TDF implementation as the reference case for
  backend-specific optimization, but do not let its current structure force a
  public or architecture-wide redesign in this phase.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/io/eic.rs`: already holds the shared query preparation, portable lazy
  extraction loop, and result initialization. This should remain the semantic
  baseline and reusable fallback path.
- `src/io/tdf/reader.rs`: already contains `prepare_eic_queries` and
  `extract_eics_fast`, plus frame metadata and native-coordinate conversion.
  This is the main backend-specific optimization surface for phase 3.
- `src/io/infer_format/dispatch.rs`: already forwards the shared EIC trait
  through `MZReaderType`, so public multi-format dispatch does not need a new
  API layer.
- `src/io/infer_format/mod.rs`: already contains `manual_extract` and
  reader-facing EIC regression checks, making it the natural place for
  integration-level semantic comparisons.

### Established Patterns
- Reader capabilities in `mzdata` are exposed as traits implemented by concrete
  readers and then forwarded through dispatch enums.
- The codebase prefers additive, backend-local optimizations over widening the
  public API for a single format.
- `DetailLevel::Lazy` and per-reader local execution are already the accepted
  portability pattern, so optimized backends should align with that mental
  model instead of introducing a parallel public access style.

### Integration Points
- Primary work will likely touch `src/io/eic.rs`, `src/io/tdf/reader.rs`, and
  `src/io/infer_format/mod.rs`, with possible small dispatch adjustments in
  `src/io/infer_format/dispatch.rs`.
- The TDF `ExtractedIonChromatogramSource` implementation is the main place
  where backend-specific routing behavior can be clarified while preserving the
  shared trait contract.
- Compatibility coverage should include both the EIC entry point and existing
  spectrum/frame reader operations so `ARCH-02` is proven rather than assumed.

</code_context>

<deferred>
## Deferred Ideas

- A general cross-format analytical index layer beside `Spectrum`
- Public TIC extraction API
- Public DT extraction API
- Higher-dimensional analytical slice APIs
- Backend-specific public tuning knobs or exposed "fast path" controls
- Documentation and real-dataset feasibility work reserved for phase 4

</deferred>

---
*Phase: 03-additive-backend-integration*
*Context gathered: 2026-03-25*
