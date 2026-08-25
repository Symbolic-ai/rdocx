# Current Sprint, S57

**Milestone**: M20 Fidelity at scale.

**Goal**: measure the Word renderer against documents nobody in this project
wrote. Establish a pinned external Word corpus, compare every rendered page
against a reviewed oracle, and set explicit memory and throughput limits for a
thousand-page document before S58 changes shaping and line breaking.

## Spec references

- `docs/hld/03-architecture.md`, for the Word facade, paginator, shared layout
  boundary, cache ownership, and dependency direction exercised by corpus and
  performance runs.
- `docs/hld/08-rendering-spec.md`, for deterministic `LayoutResult` and
  `PageFrame` output, fixed-page rendering behavior, and the pixel comparison
  boundary used by the Word SSIM harness.
- `docs/hld/12-testing-strategy.md`, for the external-corpus exception,
  deterministic-font requirement, source and oracle discipline, SSIM trend
  references, hard gates, and complete-coverage failure policy.
- `docs/hld/14-development-backlog.md`, for the M20 fidelity goal and the exact
  F-196 corpus, F-197 SSIM, and F-201 large-document performance contracts.
- `docs/hld/15-build-and-toolchain.md`, for pinned test runtimes, deterministic
  fonts, fetched-corpus CI ownership, dependency policy, and package-wide
  verification.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-196 | Word corpus | M | in-progress | codex |
| F-201 | Large document performance | L | in-progress | codex |
| F-197 | Word SSIM harness | L | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-196 and F-201 have no dependencies and may proceed independently with
exclusive ownership of corpus acquisition and performance measurement. F-197
depends on F-196 because its page comparisons require the verified corpus and
its provenance records. The completed F-197 gate protects S58, where
hyphenation and complex-script shaping may intentionally move rendered pixels.

## Definition of done for this sprint

- The Word corpus fetcher retrieves the reviewed business-letter, report,
  form, legal-revision, and multi-script documents, verifies every checksum,
  and refuses missing, changed, or unlicensed inputs.
- Every corpus page renders in deterministic font mode and is compared against
  the pinned oracle with complete coverage, per-page SSIM reporting, a reviewed
  trend reference, and an explicit hard-gate policy.
- A deliberate layout perturbation moves the SSIM result enough to prove the
  harness detects visible regressions.
- A source-built thousand-page document paginates and renders within the
  declared memory ceiling and throughput floor without unbounded retained
  state.
- Corpus, oracle, and performance inputs remain outside published crate
  archives, and no new reverse dependency enters the workspace graph.
- The full workspace gate, package checks, and deterministic hash harness pass
  without an unexplained baseline change.
