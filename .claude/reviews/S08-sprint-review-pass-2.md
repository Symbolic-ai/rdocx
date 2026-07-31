# S08 sprint review, pass 2

**Reviewed**: `sprint/s08` at `122b4f31` against merge base `01e1eb3e`,
31 files, 3,469 changed lines, crates: `oxml-pdf`, `rdocx`, `rdocx-pdf`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass 1 resolution

Pass 1 S1 is resolved. The rendering HLD now says that `rdocx-pdf` remains
dependency-separate while carrying the approved F-039 writer mirror until
F-046 at `docs/hld/08-rendering-spec.md:11`. The same HLD no longer lists the
completed per-element Y flip among the remaining defects at
`docs/hld/08-rendering-spec.md:138`.

## Milestone gate

The M5 gate is: "golden-PNG diffs of the whole sample corpus show zero pixel
changes" (`docs/hld/14-development-backlog.md:322`). It holds against the
reviewed F-039 baseline. The integrated full verification at `103b47f5`
observed exact equality for all seven page-one RGBA buffers under
`pdftoppm version 26.01.0`. The only later implementation-adjacent change is
the pass 1 HLD correction. The manifest still records that rasteriser and the
non-empty F-039 review reason at `scripts/golden_pixel_manifest.json:3` and
`scripts/golden_pixel_manifest.json:7`.

The pre-update evidence is exact rather than tolerant. The four approved pixel
coordinates and before-and-after values are recorded at
`.claude/plans/F-039-design.md:52`, and the manifest diff changes only the
`invoice` and `quote` digests. Normal check mode still compares every decoded
RGBA digest exactly at `scripts/golden_png_harness.py:446`, while the injected
one-pixel regression is exercised at `scripts/golden_png_harness.py:486`.

The operator gate is also implemented directly. The staged writer emits the
single page transform at `crates/oxml-pdf/src/writer.rs:402`, restores it at
`crates/oxml-pdf/src/writer.rs:509`, and tests the page, text, image, primitive,
and annotation operators at `crates/oxml-pdf/src/writer.rs:730`. The shipped
writer carries the same focused change. The existing 28-entry hash baseline
has no sprint diff, consistent with the integrated unchanged-harness result.

## Not found

- `interaction`: the deterministic PDF facade uses the shipped writer, while
  the staged writer carries the same page, text, image, primitive, and
  annotation transforms. No jointly incorrect F-ID interaction was found.
- `duplication`: no helper was independently added under another name. The two
  writer copies are the approved temporary migration state through F-046.
- `layering`: `oxml-pdf` has only `oxml-layout`, `oxml-media`, and external PDF
  implementation dependencies at `crates/oxml-pdf/Cargo.toml:15`. It has no
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: the hash baseline is unchanged. The golden manifest carries the
  reviewed F-039 reason, records only the two declared digest changes, and
  retains exact comparison with no tolerance.
- `gate`: the eight moved backend cases, deterministic PDF facade, exact
  seven-buffer comparison, deliberate one-pixel rejection, writer operator
  assertions, and unchanged hash harness cover the sprint definition of done.
- `docs`: the plans, CURRENT_SPRINT, rendering and testing HLD, migration
  boundary, and exact four-pixel evidence describe the implemented state.
- `deps`: no new external dependency was added. Every new workspace dependency
  is consumed directly by `oxml-pdf`.
- `surface`: `Document::to_pdf_deterministic` at
  `crates/rdocx/src/document.rs:2163` is the only added released public API and
  is required by F-038's deterministic PDF gate.
- `publication`: released crate manifests and dependency edges are unchanged.
  `oxml-pdf` remains at version 0.0.0 with publication disabled at
  `crates/oxml-pdf/Cargo.toml:4`.
- `ledgers`: CURRENT_SPRINT marks all three stories done at
  `docs/sprints/CURRENT_SPRINT.md:30`. BACKLOG, SPRINT_TRACKER, and AS_BUILT
  agree on F-037 through F-039 and their S08 completion.
