# F-104, SSIM fidelity harness

**Status**: approved
**Sprint**: S25
**Size**: L
**Depends on**: F-102

## Problem

The M10 gate requires at least 0.95 SSIM on at least 80 percent of corpus
slides, with every slide rendering without panic or a dropped shape at
`docs/hld/12-testing-strategy.md:148`. The repository has a pinned 50-deck
corpus and deterministic PNG machinery, but no whole-deck render command, no
LibreOffice comparison harness, and no CI fidelity job. The current
`rpptx-render::layout_presentation` also constructs the normal system-font
manager at `crates/rpptx-render/src/lib.rs:179`, which is unsuitable for a
recorded baseline.

The local machine has `pdftoppm` but no `libreoffice` or `soffice` executable.
The oracle therefore needs an approved, exact installation before the gate can
be observed locally.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "The measurable bar".
- `docs/hld/08-rendering-spec.md`, the renderer pipeline and visible fallback
  invariant.
- `docs/hld/12-testing-strategy.md`, "The render fidelity gate".
- `docs/hld/14-development-backlog.md`, "F-104, SSIM fidelity harness" and the
  M10 end-of-milestone gate.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".
- `.claude/skills/differential-testing.md`, external oracle rules.

## Approach

Add `layout_presentation_deterministic` beside the normal renderer entry point.
It uses the bundled-font-only `FontManager` and otherwise shares the same
private page-lowering implementation, so the fidelity path cannot silently
diverge from normal rendering.

Add `crates/rpptx/examples/render_deck.rs` as the concrete corpus driver. It
opens each package, resolves every slide, relationship scope, media item,
theme, and table-style list into one `RenderInput`, renders every page at 150
dpi, and writes deterministic PNGs plus a tab-separated manifest containing
source leaf counts, resolved shape counts, diagnostics, and output paths. It
fails on any panic, missing page, or dropped bounded source shape.

Add `scripts/pptx_ssim_harness.py` as the orchestrator and metric owner. It
verifies the 50-deck manifest, invokes the Rust corpus driver once, records and
checks the exact LibreOffice version, renders the oracle through LibreOffice
and a pinned 150 dpi PDF raster path, decodes PNGs with the existing
golden-harness decoder, and computes a documented deterministic SSIM score.
The script reports per-slide scores and corpus totals, fails if any slide is
missing or dropped, and enforces 0.95 on at least 80 percent. Embedded unit
tests pin identical, changed, dimension-mismatch, and threshold behavior.

Add a dedicated CI job that fetches the pinned corpus, installs the exact
recorded LibreOffice and raster tools, and runs the harness in required mode.
Record a one-time M10 PowerPoint spot-check over representative low, median,
and high SSIM outputs. LibreOffice differences remain review-required rather
than becoming an unexplained allowlist.

## Rejected alternatives

- Use system fonts for convenience. The resulting scores would not be
  reproducible across machines.
- Commit oracle PNGs. The corpus is external, binary fixtures are prohibited,
  and LibreOffice output belongs to the pinned executable version.
- Put the harness inside a published crate. The external oracle is test
  infrastructure and must not become a crate dependency.
- Compare only the first slide of each deck. The gate is defined over every
  corpus slide.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `identical_images_have_ssim_one` | The metric gives exactly 1.0 for equal decoded pixels |
| unit | `pixel_changes_lower_ssim` | A controlled difference produces a stable lower score |
| regression | `dimension_mismatch_is_a_hard_failure` | Misaligned oracle output cannot be scored or hidden |
| unit | `coverage_threshold_requires_eighty_percent` | The aggregate gate uses the exact 0.95 and 80 percent boundaries |
| integration | `all_corpus_slides_render_without_panic_or_dropped_shape` | The hard half of the backlog gate over the required 50-deck corpus |
| differential | `corpus_render_fidelity_meets_ssim_target` | At least 80 percent of all slides score at least 0.95 against the pinned LibreOffice oracle |

The test gate is at least 0.95 SSIM on at least 80 percent of slides, and 100
percent render without panic or dropped shape.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. The corpus driver uses deterministic fonts
  exclusively, and no system-font baseline is recorded.
- External oracle comparison: read
  `.claude/skills/differential-testing.md`. Pin and record the exact
  LibreOffice version, 150 dpi raster tool version, metric definition, and
  triage for every known divergence.
- New files: read the structural rules in `CLAUDE.md`. The two proposed files
  have distinct existing consumers today, the run-sprint gate and the CI
  fidelity job. Explicit approval is required before creation.
- Crate dependency graph: read `docs/hld/03-architecture.md`. Keep the oracle
  outside every crate and add only existing one-way development dependencies
  needed by the unpublished example.

Extra checks are the harness unit tests, required 50-deck corpus run, exact
oracle-version assertion, deterministic-font assertion, and the M10 manual
PowerPoint spot-check.

## Hash harness

Expected to be unchanged. The existing 28-entry Word harness does not consume
PresentationML rendering.

## Implementation checklist

- [ ] Add a deterministic whole-presentation layout entry point without
  changing normal font discovery.
- [ ] Add the approved whole-deck corpus renderer and source-to-resolved shape
  accounting.
- [ ] Add the approved version-pinned LibreOffice SSIM harness and unit tests.
- [ ] Enforce 150 dpi, 0.95 SSIM, 80 percent coverage, and 100 percent complete
  rendering.
- [ ] Add the required CI fidelity job with pinned tools and corpus.
- [ ] Triage below-threshold slides and fix in-scope renderer defects.
- [ ] Record the representative M10 PowerPoint spot-check.

## Open questions

None. The two named harness files are approved, as are an exact pinned
LibreOffice and 150 dpi raster tool installation and a representative low,
median, and high SSIM PowerPoint spot-check for M10.
