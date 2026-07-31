# S08 sprint review, pass 1

**Reviewed**: `sprint/s08` at `7dd2e352` against merge base `01e1eb3e`,
30 files, 3,393 changed lines, crates: `oxml-pdf`, `rdocx`, `rdocx-pdf`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, the rendering HLD still describes the completed CTM change as pending

`docs/hld/08-rendering-spec.md:11`

`docs/hld/08-rendering-spec.md:143`

The HLD says the released `rdocx-pdf` backend remains unchanged and still lists
the per-element Y flip as a latent defect. The integrated F-039 implementation
mirrors the global page CTM into that backend, as the same HLD later specifies
at `docs/hld/08-rendering-spec.md:153`. This gives F-040 and F-046 conflicting
current-state instructions. Update the seam description to say that the
released backend remains dependency-separate but received the approved F-039
writer change, and remove or reclassify the resolved Y-flip row.

## Nice-to-have

None.

## Milestone gate

The M5 gate is: "golden-PNG diffs of the whole sample corpus show zero pixel
changes" (`docs/hld/14-development-backlog.md:322`). It holds against the
reviewed F-039 baseline. The full integrated verification recorded at
`103b47f5` found all seven page-one RGBA buffers exact under `pdftoppm version
26.01.0`. The manifest records that rasteriser and the reviewed F-039 reason at
`scripts/golden_pixel_manifest.json:3` and
`scripts/golden_pixel_manifest.json:7`. The F-039 review also records the exact
seven-buffer pass and injected-pixel rejection at
`.claude/reviews/F-039-all-pass-2.md:34`.

The sprint's additional stability gate also holds. The same integrated
verification found all 28 hash-harness entries unchanged, and
`scripts/hash_baseline.json` has no sprint diff. The old golden manifest changed
only the `invoice` and `quote` digests plus the non-empty review reason. The
approved four pixel coordinates and values are recorded at
`.claude/plans/F-039-design.md:52`.

## Not found

- `interaction`: the deterministic PDF facade drives the shipped writer that
  received the mirrored CTM change, and the staged writer carries the same
  matrices. No jointly incorrect F-ID interaction was found.
- `duplication`: no helper was independently reimplemented under another name.
  The two writer copies are the approved temporary migration state through
  F-046, and their focused differences are limited to layout types, staged
  unsupported arms, and staged tests.
- `layering`: `oxml-pdf` depends on `oxml-layout`, `oxml-media`, and external
  renderer dependencies only. No `rdocx-*` or `rpptx-*` edge was added.
- `harness`: no hash baseline changed. The golden manifest delta is limited to
  the two declared samples and carries the reviewed F-039 reason.
- `gate`: no untested milestone-gate assertion or undisclosed rendering delta
  was found.
- `deps`: no new external dependency was introduced. Each new workspace edge
  has a named `oxml-pdf` consumer.
- `surface`: `Document::to_pdf_deterministic` is the only added released public
  API, and F-038 explicitly requires it for deterministic PDF generation.
- `publication`: released crate manifests and versions are unchanged.
  `oxml-pdf` remains at 0.0.0 with `publish = false`.
- `ledgers`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and the three AS_BUILT
  entries agree that F-037 through F-039 are complete in S08.
