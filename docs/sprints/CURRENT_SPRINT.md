# Current Sprint, S08

**Milestone**: M5 PDF backend.

**Goal**: Stage the shared PDF backend and move it to one global coordinate
transform with zero pixel change. Establish the deterministic golden-PNG gate
before the coordinate-system rewrite so any regression has one possible cause.

## Spec references

- `docs/hld/03-architecture.md`, for the `oxml-pdf` boundary, its exclusive
  consumption of `LayoutResult`, and the dependency rule for staged crates.
- `docs/hld/08-rendering-spec.md`, for the global CTM, upright text matrix,
  image matrix, unchanged annotation path, and pixel-comparison requirement.
- `docs/hld/11-migration-plan.md`, for staging `oxml-pdf` while released rdocx
  stays unchanged and deferring the facade cutover until shared publication.
- `docs/hld/12-testing-strategy.md`, for deterministic golden-PNG comparison,
  the injected one-pixel failure proof, and the `oxml-pdf` backend test floor.
- `docs/hld/13-risks-and-open-questions.md`, for the coordinate-system risk and
  the requirement to isolate the flip before any PowerPoint rendering code.
- `docs/hld/14-development-backlog.md`, for the F-037 through F-039 contracts,
  dependencies, sizes, test gates, and the M5 zero-pixel-change milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for deterministic font mode, WASM
  constraints, staged versioning, packaging, and publication boundaries.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-037 | Create oxml-pdf | S | pending | - |
| F-038 | Golden-PNG harness | M | pending | - |
| F-039 | Global CTM flip | L | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-037 first stages the backend on the shared layout and media crates without
changing released rdocx. F-038 then records and proves the deterministic pixel
comparison gate. F-039 lands last as its own reviewable behavioural commit,
with that gate available to prove the operator rewrite causes no pixel change.

## Definition of done for this sprint

- `oxml-pdf` is an isolated staged copy wired to `oxml-layout` and
  `oxml-media`, its duplicated header parsers are removed, and its eight moved
  tests pass.
- The deterministic golden-PNG harness passes on an unmodified sample corpus
  and fails when a one-pixel offset is deliberately injected.
- The PDF writer emits one page-level `q 1 0 0 -1 0 H cm`, uses an upright text
  matrix and negative-height image matrix, and leaves link annotations outside
  the content stream unchanged.
- Golden-PNG comparisons across the whole sample corpus show zero pixel change,
  while the existing 28-entry hash harness remains unchanged.
- Released rdocx manifests and dependencies stay unchanged. Its additive
  deterministic PDF facade and mirrored writer rewrite preserve every sample
  pixel, while staged crates remain at 0.0.0 with publication disabled.
- The full workspace, no-default-features, WASM, documentation, package, and
  supply-chain gates pass without publishing any crate.
