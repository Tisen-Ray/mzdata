# Requirements: mzdata Analytical Access

**Defined:** 2026-03-25
**Core Value:** A caller can extract targeted analytical traces through one
clean API without loading the whole file or writing format-specific extraction
code.

## v1 Requirements

### EIC API

- [ ] **EIC-01**: Caller can request an extracted ion chromatogram through one
  public API that works across supported `MZReader` backends.
- [ ] **EIC-02**: The shared EIC query type supports at least `m/z` range and
  optional RT, MS level, and mobility filtering.
- [ ] **EIC-03**: The default cross-format EIC path reads spectra on demand
  instead of materializing the entire source file up front.
- [ ] **EIC-04**: When ordered `m/z` arrays are available, the shared EIC path
  uses binary search to restrict work to the matching interval instead of
  linearly scanning the full array.
- [ ] **EIC-05**: Format-specific optimizations, such as Bruker TDF fast paths,
  remain hidden behind the same public EIC interface.

### Architecture

- [ ] **ARCH-01**: The new analytical extraction code fits naturally into the
  existing trait and module structure without making the reader hierarchy harder
  to understand.
- [ ] **ARCH-02**: Existing spectrum and frame access APIs continue to behave as
  before after the EIC interface is introduced.
- [ ] **ARCH-03**: The resulting file layout and trait structure are clear
  enough that future TIC/DT work can extend them without a redesign.

### Documentation

- [ ] **DOCS-01**: Project documentation explains the chosen analytical access
  model: per-spectrum lazy loading plus spectrum-internal binary search.
- [ ] **DOCS-02**: Documentation clearly distinguishes current EIC scope from
  deferred TIC/DT and broader analytical slicing work.
- [ ] **DOCS-03**: Documentation examples for the new EIC API pass doc testing
  or equivalent validation in the library's documentation workflow.

### Verification

- [ ] **TEST-01**: The codebase includes at least one small validation path that
  exercises the shared EIC API against a real dataset.
- [ ] **TEST-02**: A lightweight test or validation flow confirms feasibility
  using `C:\Users\ray\Desktop\20241005-TDP-CYTO-C-1.raw`.
- [ ] **TEST-03**: The EIC implementation is checked against at least one simple
  expected-result path so maintainers can detect obvious regressions.

## v2 Requirements

### Additional Traces

- **TIC-01**: Caller can extract total ion chromatograms through the same
  analytical access layer.
- **DT-01**: Caller can extract drift-time traces without introducing a new
  public access model.
- **SLCE-01**: Caller can request richer analytical slices, such as RT-by-mobility
  or mobility-filtered trace views.

### Performance

- **PERF-01**: The project has a broader benchmark suite across multiple formats
  and representative datasets.
- **PERF-02**: More formats can exploit deeper internal optimizations while
  keeping the public analytical API unchanged.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Public frame/scan-native analytical API | Conflicts with the goal of flattening format differences |
| Disk-level sub-spectrum byte-range loading | Too format-specific for the first milestone |
| Full TIC and DT feature set | Deferred until EIC design and tests are stable |
| Large benchmark harness across many raw datasets | Not required to prove v1 feasibility |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| EIC-01 | Phase 1 - Public EIC Surface | Pending |
| EIC-02 | Phase 1 - Public EIC Surface | Pending |
| ARCH-01 | Phase 1 - Public EIC Surface | Pending |
| EIC-03 | Phase 2 - Portable Spectrum EIC Engine | Pending |
| EIC-04 | Phase 2 - Portable Spectrum EIC Engine | Pending |
| TEST-03 | Phase 2 - Portable Spectrum EIC Engine | Pending |
| EIC-05 | Phase 3 - Additive Backend Integration | Pending |
| ARCH-02 | Phase 3 - Additive Backend Integration | Pending |
| ARCH-03 | Phase 3 - Additive Backend Integration | Pending |
| DOCS-01 | Phase 4 - Docs and Feasibility Validation | Pending |
| DOCS-02 | Phase 4 - Docs and Feasibility Validation | Pending |
| DOCS-03 | Phase 4 - Docs and Feasibility Validation | Pending |
| TEST-01 | Phase 4 - Docs and Feasibility Validation | Pending |
| TEST-02 | Phase 4 - Docs and Feasibility Validation | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0

---
*Requirements defined: 2026-03-25*
*Last updated: 2026-03-25 after roadmap traceability mapping*
