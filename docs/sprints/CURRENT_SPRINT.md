# Current Sprint, S06

**Milestone**: M4 Layout primitives.

**Goal**: Stage the format-neutral layout output, font, and line-breaking types
inside an isolated `oxml-layout` crate. Resolve the one genuine API boundary in
`line.rs` and add affine transforms without changing released rdocx consumers
or their rendered output.

## Spec references

- `docs/hld/03-architecture.md`, for the format-neutral layout boundary, the
  acyclic dependency rule, and the unpublished 0.0.0 staging policy.
- `docs/hld/08-rendering-spec.md`, for the existing output seam, the 2x3 affine
  `Transform` contract, and the layout-specific regression obligations.
- `docs/hld/11-migration-plan.md`, for the staging order, the exact docx types
  that `line.rs` must replace, and the unchanged-output migration rule.
- `docs/hld/12-testing-strategy.md`, for transform composition, font-manager,
  no-default-features, workspace, hash, and packaging gates.
- `docs/hld/13-risks-and-open-questions.md`, for the bundled-font archive risk
  that the staged crate must preserve and measure.
- `docs/hld/14-development-backlog.md`, for the F-029 through F-031 contracts,
  dependencies, sizes, and story test gates.
- `docs/hld/15-build-and-toolchain.md`, for the `system-fonts` feature,
  bundled-font packaging, archive-size limit, and publication boundary.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-029 | Create oxml-layout | M | in-progress | codex |
| F-030 | Decouple line.rs | L | in-progress | codex |
| F-031 | Transform | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-029 creates the staged crate and blocks both later stories. F-030 then owns
the high-drift `line.rs` API decoupling as its own reviewed change. F-031 can
proceed independently after F-029 because its affine implementation does not
depend on the line-breaking conversion.

## Definition of done for this sprint

- `oxml-layout` exists at version 0.0.0 with publication disabled, and released
  rdocx manifests and consumers remain unchanged.
- The staged output, font, bundled-font, and error implementation passes its
  copied tests, while `Document::load_fonts_from_dir` remains unchanged.
- Staged `line.rs` uses owned `TabStop`, `Align`, `TabAlign`, `Underline`, and
  `LineSpacing` types plus explicit wrapping, with all 11 rewritten tests green.
- The 2x3 affine `Transform` supports rotation, composition, application,
  identity checks, and rectangle bounds with PDF `cm` composition order proven
  against a hand-computed matrix.
- The staged crate passes its normal and no-default-features paths, package and
  supply-chain gates, and remains within the crates.io archive limit.
- The full workspace passes with all 28 hash-harness entries unchanged, no
  development crate is published, and no released rdocx dependency changes.
