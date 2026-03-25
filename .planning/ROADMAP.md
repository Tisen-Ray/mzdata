# Roadmap: mzdata Analytical Access

## Overview

v1 turns the existing EIC direction in `mzdata` into a clean public access path
for brownfield evolution: one shared spectrum-oriented API, a portable lazy
extraction strategy based on per-spectrum loading plus spectrum-internal binary
search, additive backend optimizations behind that surface, and lightweight
documentation and validation. TIC, DT, and broader analytical slicing remain
explicitly deferred until this EIC foundation is stable.

## Phases

- [ ] **Phase 1: Public EIC Surface** - establish the shared public query and
  trait layout for cross-format callers.
- [ ] **Phase 2: Portable Spectrum EIC Engine** - implement the common lazy,
  binary-search extraction path and a simple regression anchor.
- [ ] **Phase 3: Additive Backend Integration** - preserve compatibility while
  hiding backend-specific accelerations behind the same API.
- [ ] **Phase 4: Docs and Feasibility Validation** - document the chosen model
  and prove it on a small real RAW dataset.

## Phase Details

### Phase 1: Public EIC Surface
**Goal:** Introduce the unified public EIC entry points and query contract in a
trait/module layout that feels native to `mzdata`'s existing reader
architecture.
**Depends on:** Nothing (first phase)
**Requirements:** [EIC-01, EIC-02, ARCH-01]
**Success Criteria** (what must be TRUE):
1. Callers can request an EIC through one public API on `MZReader` without
   choosing a format-specific path.
2. The shared query type supports `m/z` range plus optional RT, MS level, and
   mobility filtering.
3. The new analytical extraction traits/modules sit in a clear additive
   location that preserves the current reader hierarchy.
**Plans:** TBD

### Phase 2: Portable Spectrum EIC Engine
**Goal:** Make the default cross-format EIC implementation match the chosen
portable strategy: lazy spectrum access plus spectrum-internal binary search
over ordered `m/z` arrays.
**Depends on:** Phase 1
**Requirements:** [EIC-03, EIC-04, TEST-03]
**Success Criteria** (what must be TRUE):
1. The default shared EIC path reads spectra on demand instead of materializing
   the full file up front.
2. When ordered `m/z` arrays are available, the shared path narrows the
   matching interval by binary search before summing intensities.
3. A lightweight expected-result regression check exists for the shared
   extraction logic so obvious query/filtering mistakes are detectable.
**Plans:** TBD

### Phase 3: Additive Backend Integration
**Goal:** Hide format-specific accelerations behind the same public EIC surface
while preserving existing spectrum/frame behavior and future extension seams.
**Depends on:** Phase 2
**Requirements:** [EIC-05, ARCH-02, ARCH-03]
**Success Criteria** (what must be TRUE):
1. Backend-specific optimizations such as the Bruker TDF fast path remain
   reachable only through the shared public EIC API.
2. Existing spectrum and frame access APIs continue to behave as before after
   EIC integration.
3. The resulting trait and file layout leaves a clear additive path for future
   TIC/DT work without redesigning the public model.
**Plans:** TBD

### Phase 4: Docs and Feasibility Validation
**Goal:** Document the chosen EIC model, state the milestone boundary clearly,
and validate feasibility with lightweight examples/tests plus a small RAW
dataset.
**Depends on:** Phase 3
**Requirements:** [DOCS-01, DOCS-02, DOCS-03, TEST-01, TEST-02]
**Success Criteria** (what must be TRUE):
1. Documentation explains the per-spectrum lazy-loading plus
   spectrum-internal binary-search model and its portability tradeoffs.
2. Documentation explicitly marks TIC, DT, and higher-dimensional analytical
   slicing as deferred beyond v1.
3. Public EIC examples pass doc testing or the repository's equivalent
   documentation validation flow.
4. The codebase includes a small real-data validation path that exercises the
   shared EIC API.
5. Maintainers can run a lightweight feasibility check using
   `C:\Users\ray\Desktop\20241005-TDP-CYTO-C-1.raw`.
**Plans:** TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Public EIC Surface | 0/TBD | Not started | - |
| 2. Portable Spectrum EIC Engine | 0/TBD | Not started | - |
| 3. Additive Backend Integration | 0/TBD | Not started | - |
| 4. Docs and Feasibility Validation | 0/TBD | Not started | - |
