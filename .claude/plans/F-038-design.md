# F-038, Golden-PNG harness

**Status**: approved
**Sprint**: S08
**Size**: M
**Depends on**: F-037, F-001

## Problem

`scripts/hash_harness.py:121` hashes PNG file bytes emitted directly by the
tiny-skia backend. It does not rasterise PDFs, compare decoded pixels, or
exercise the PDF coordinate operators that F-039 changes. A PDF-byte baseline
would also be wrong because F-039 deliberately rewrites the operator stream
while preserving its visual result.

The existing sample generator calls the normal-font `Document::to_pdf()` path,
so its PDFs are not valid deterministic baselines. The repository has a
deterministic PNG facade at `crates/rdocx/src/document.rs:2269`, but no matching
deterministic PDF facade.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The PDF backend".
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and "The hash
  harness".
- `docs/hld/13-risks-and-open-questions.md`, "R2, the PDF coordinate-system
  flip".
- `docs/hld/14-development-backlog.md`, "F-038, Golden-PNG harness".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".

## Approach

Add `scripts/golden_png_harness.py` and a readable JSON manifest containing
the width, height, and SHA-256 digest of decoded RGBA pixels for page one of
each of the seven existing samples. Generated PDFs and PNGs remain ignored.
The harness has check and reviewed-update modes, reports the first differing
sample precisely, and rejects an update without a non-empty reason.

Generate the PDFs through a minimal deterministic PDF facade that reuses the
existing cached deterministic layout and the existing `rdocx_pdf::render_to_pdf`
call. Rasterise at a fixed DPI with a recorded Poppler `pdftoppm` version, then
decode and compare pixel buffers rather than PNG bytes. Poppler is test
infrastructure only and is never a crate dependency.

Unit-test the comparison and update-reason logic with code-built pixel buffers.
The story gate also copies one generated image into a temporary directory,
changes exactly one decoded pixel, and proves check mode fails for that sample.
No binary fixture is committed.

## Rejected alternatives

- Reuse the hash harness PNG digests. Those images bypass the PDF writer and
  compare encoded bytes rather than pixels.
- Compare PDF bytes. The global CTM story intentionally changes those bytes.
- Record system-font output. That baseline cannot reproduce across machines.
- Commit golden PNG files. The testing strategy forbids binary fixtures, and a
  pixel manifest is smaller and reviewable.
- Add Poppler below `crates/*/src`. External rasterisation belongs only in test
  infrastructure.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `pixel_comparison_reports_dimension_and_digest_changes` | Dimension and RGBA differences identify the exact sample. |
| unit | `update_requires_a_non_empty_reason` | Baseline writes cannot occur without an audit reason. |
| golden, gate | `unmodified_sample_corpus_matches_the_pixel_manifest` | All seven deterministic PDF renders match decoded page-one pixels. |
| regression, gate | `one_pixel_offset_is_rejected` | A deliberate one-pixel mutation makes the harness fail with the sample name. |
| integration | `rasterizer_version_is_recorded` | The run reports the exact Poppler version used to produce and check pixels. |

The backlog test gate is that the harness passes on an unmodified tree and
fails on an injected one-pixel offset.

## HLD impact

- `docs/hld/12-testing-strategy.md`, specify the command, deterministic PDF
  path, pixel manifest, rasteriser version record, DPI, and failure reporting.
- `docs/hld/15-build-and-toolchain.md`, add the deterministic PDF facade used
  by the golden gate alongside the existing deterministic PNG facade.

## Risk routing

- Layout and rendering baseline. Generate every PDF from bundled fonts only,
  record no system-font output, and require the existing hash harness to stay
  unchanged.
- Public API of published `rdocx`. The approved deterministic PDF method is
  additive and reuses the existing deterministic layout cache. Run the full
  package dry-run and archive-size gate without publishing.
- External rasteriser. Record the exact Poppler version, fixed DPI, exact-pixel
  metric, and zero-difference threshold. Poppler remains test infrastructure.
- New files. The approved boundary authorizes the harness script and JSON
  manifest. Add no binary fixture or extra Rust test binary.

## Hash harness

Expected to remain unchanged. This story records a distinct decoded-pixel gate
and does not alter released rendering behaviour.

## Implementation checklist

- [ ] Add the approved deterministic PDF entry point.
- [ ] Add check and reviewed-update modes to the golden harness.
- [ ] Generate the seven sample PDFs with bundled fonts only.
- [ ] Rasterise page one at the approved fixed DPI and compare decoded pixels.
- [ ] Record a readable JSON pixel manifest without binary fixtures.
- [ ] Prove the unmodified pass and injected one-pixel failure.
- [ ] Run the hash, package, prose, and rasteriser-version riders.

## Open questions

None. The deterministic PDF facade, harness script, and JSON pixel manifest
are approved. F-039 will mirror the CTM rewrite into both PDF backends so the
seven-sample gate exercises the shipped renderer.
