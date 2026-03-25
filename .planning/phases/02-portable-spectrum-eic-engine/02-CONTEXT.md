# Phase 2: Portable Spectrum EIC Engine - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase implements the default cross-format EIC engine behind the public
reader-native API introduced in Phase 1. The scope is the portable fallback:
lazy per-spectrum access, spectrum-internal binary search over ordered `m/z`
arrays, and a lightweight regression anchor. It does not add new analytical
capabilities, redesign the public API, or fold in backend-specific acceleration
work reserved for later phases.

</domain>

<decisions>
## Implementation Decisions

### Portable extraction behavior
- Keep the Phase 1 reader-native API and implement the Phase 2 engine behind
  the existing `extract_eic` / `extract_eics` surface.
- Favor the smallest viable refactor: tighten and formalize the existing
  `extract_eics_from_spectra` path instead of introducing a new abstraction
  layer in this phase.
- The generic engine should continue to iterate spectra lazily, read only the
  arrays needed for matching spectra, and avoid whole-file materialization.

### Trace shape and filtering semantics
- Preserve a stable chromatogram shape by emitting a time/intensity point for
  each spectrum that passes spectrum-level filters, even when the summed signal
  is `0`.
- Treat RT, MS level, and minimum-intensity filtering exactly as part of the
  shared portable contract.
- When a query requests mobility filtering, the generic path should require a
  portable spectrum-level mobility value. Spectra without a portable mobility
  point should be excluded rather than silently ignoring the filter.
- Do not broaden Phase 2 into frame-level or ion-mobility-dimension extraction;
  this phase stays within portable spectrum-oriented semantics.

### Binary-search strategy
- The optimization target is spectrum-internal narrowing over ordered `m/z`
  arrays using binary search / partition-point style bounds lookup before
  intensity accumulation.
- Reuse existing raw-array access when available and fall back to peak iteration
  only where the format does not expose the ordered arrays needed for the fast
  path.
- Keep the optimization local to `src/io/eic.rs` and adjacent dispatch/tests so
  later phases can add backend-specific fast paths without undoing Phase 2.

### Regression and verification
- Use lightweight regression anchors first: focused synthetic/in-memory tests
  that prove lazy-spectrum behavior, binary-search interval narrowing, and
  combined filter correctness.
- Add a tiny file-backed check only if the existing fixtures make it cheap and
  non-brittle; do not let fixture work dominate the phase.
- Verification should prove the engine does not regress into eager
  full-materialization behavior for the portable shared path.

### Success bar
- Phase 2 is done when the shared portable EIC path is clearly lazy,
  binary-search based where ordered arrays exist, and backed by regression tests
  that would catch a fallback to naive full-spectrum scanning.

### Claude's Discretion
- Small helper extraction or internal cleanup needed to make the shared EIC
  engine easier to read.
- Exact test split between `src/io/eic.rs` and nearby dispatch/module tests.
- Whether one tiny fixture-backed assertion is worth keeping once the synthetic
  regression anchor exists.

</decisions>

<specifics>
## Specific Ideas

- Prefer behavior that keeps the library elegant and additive rather than
  introducing broad new indexing or analytical-layer abstractions in this
  phase.
- The portable fallback should feel like a disciplined refinement of the
  existing EIC implementation, not a second architecture.
- Backend-specific acceleration remains important, but only as a later additive
  phase built on the same public and portable contract.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/io/eic.rs`: already contains query preparation, spectrum-level filtering,
  lazy detail-level switching, and `m/z` interval summation via
  `sum_array_range`.
- `src/io/infer_format/dispatch.rs`: already forwards the shared EIC trait
  through `MZReaderType`, so Phase 2 can stay behind the same public API.
- `MemorySpectrumSource` and current EIC tests: good fit for synthetic portable
  regression coverage without heavy fixture setup.

### Established Patterns
- `mzdata` prefers additive extensions to the existing reader hierarchy rather
  than parallel service objects.
- `DetailLevel::Lazy` is the established way to avoid eager array loading in
  shared reader code.
- The codebase already treats TDF as a richer special case while keeping a
  generic spectrum-oriented fallback for shared behavior.

### Integration Points
- Primary implementation work should stay centered in `src/io/eic.rs`.
- Dispatch visibility and public behavior checks remain in
  `src/io/infer_format/dispatch.rs` and `src/io/infer_format/mod.rs`.
- Requirement/test traceability will need to cover `EIC-03`, `EIC-04`, and
  `TEST-03` without reopening the Phase 1 public API/export work.

</code_context>

<deferred>
## Deferred Ideas

- A shared analytical index layer beside `Spectrum`
- Public TIC extraction
- Public DT extraction
- Higher-dimensional analytical slicing
- Backend-specific acceleration work beyond the portable fallback

</deferred>

---
*Phase: 02-portable-spectrum-eic-engine*
*Context gathered: 2026-03-25*
