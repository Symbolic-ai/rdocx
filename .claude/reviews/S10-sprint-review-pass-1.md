# S10 sprint review, pass 1

**Reviewed**: `sprint/s10` at `43d3be8` against merge base `57aeaa3`,
16 files, 2,202 changed lines, crates: `oxml-pdf`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, partial-range PDF stops are stretched across the full gradient axis

`crates/oxml-pdf/src/writer.rs:786`

The shading and stitching domains are set to the first and last normalized stop
offsets. PDF maps the start and end coordinates to those domain endpoints, so a
gradient whose stops are at 0.25 and 0.75 transitions across the whole axis
instead of holding the first colour to 0.25 and the last colour after 0.75. The
raster backend passes the actual offsets to tiny-skia, so F-043 and F-045 now
disagree on a valid shared paint. Normalize the PDF function graph over the
full `[0, 1]` domain, preserving endpoint colours outside the supplied stop
range, and add a regression that covers stops which do not begin at zero or end
at one.

## Should-fix

### S1, the sprint goal overstates the supported paint and element surface

`docs/sprints/CURRENT_SPRINT.md:5`

The goal says the backend covers every paint and element form exposed by the
shared layout types, but tile paint and group effects intentionally remain
unsupported at `crates/oxml-pdf/src/raster.rs:314` and
`docs/hld/08-rendering-spec.md:217`. Narrow the goal to the forms owned by
F-043 and F-045 so the sprint contract agrees with the implemented boundary.

## Nice-to-have

None.

## Milestone gate

The M5 gate is: "golden-PNG diffs of the whole sample corpus show zero pixel
changes" (`docs/hld/14-development-backlog.md:322`). Consolidated verification
at `500d328` compared all seven page-one RGBA buffers exactly at 150 DPI under
the recorded `pdftoppm version 26.01.0` configuration in
`scripts/golden_pixel_manifest.json:3`. The deliberate one-pixel `quote`
mutation failed and named only that sample. The 28-entry hash harness also
matched during this review.

The sprint-specific sampled gates are present at
`crates/oxml-pdf/src/writer.rs:1741`,
`crates/oxml-pdf/src/raster.rs:728`, and
`crates/oxml-pdf/src/raster.rs:751`. They cover a rotated PDF gradient, a
rotated raster path, and deterministic dashed-line gaps. The partial-stop case
in B1 remains outside that evidence, so the sprint is not ready to merge.

## Not found

- `duplication`: PDF resource construction and tiny-skia paint conversion use
  backend-specific helpers. Their shared offset rules are independently tested,
  and no competing helper was introduced within either backend.
- `layering`: `oxml-pdf` depends on `oxml-layout`, `oxml-media`, and external
  implementation crates only at `crates/oxml-pdf/Cargo.toml:15`. It has no
  `rdocx-*` or `rpptx-*` edge.
- `harness`: neither `scripts/hash_baseline.json` nor
  `scripts/golden_pixel_manifest.json` changed in S10. The integrated exact
  comparisons passed.
- `gate`: aside from B1, the focused tests, exact raster comparison,
  injected-pixel rejection, workspace gate, package dry-run, archive-size
  check, and supply-chain check cover the sprint definition of done.
- `deps`: no manifest or lockfile changed, so S10 introduced no dependency.
- `surface`: all new writer and raster helpers are private. No public API was
  added.
- `publication`: `oxml-pdf` remains at version 0.0.0 with `publish = false` at
  `crates/oxml-pdf/Cargo.toml:4`, and no crate was published.
- `ledgers`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree that
  F-043 and F-045 are complete in S10.
