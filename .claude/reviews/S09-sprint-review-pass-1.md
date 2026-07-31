# S09 sprint review, pass 1

**Reviewed**: `sprint/s09` at `dc75ca81` against merge base `06b28f14`,
20 files, 1,833 changed lines, crates: `oxml-pdf`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, the recursion-hazard table points to obsolete writer lines

`docs/hld/08-rendering-spec.md:129`

`docs/hld/08-rendering-spec.md:130`

The table still locates XObject registration at `writer.rs:69` and link
annotations at `writer.rs:99` and `:355`. Those lines now belong to alpha-state
and unrelated writer code after the integrated S09 changes. Replace these
brittle line references with the stable function or pass names so the updated
current-state HLD directs readers to the implemented collection logic.

## Nice-to-have

None.

## Milestone gate

The M5 gate is: "golden-PNG diffs of the whole sample corpus show zero pixel
changes" (`docs/hld/14-development-backlog.md:322`). It holds against the
reviewed F-039 baseline. Consolidated verification at `db639053` found all
seven page-one RGBA buffers exact under `pdftoppm version 26.01.0`, which the
manifest records at `scripts/golden_pixel_manifest.json:3`. The deliberate
one-pixel `quote` mutation failed and named only that sample.

The S09 gates also hold. The group, path, alpha, and nested collection tests are
present at `crates/oxml-pdf/src/writer.rs:1106`,
`crates/oxml-pdf/src/writer.rs:1226`,
`crates/oxml-pdf/src/writer.rs:1352`, and
`crates/oxml-pdf/src/writer.rs:1444`. The 28-entry hash baseline has no sprint
diff. Packaging ran with `cargo publish --workspace --dry-run`, every archive
was below 10 MiB, and no upload occurred.

## Not found

- `interaction`: path geometry is shared by visible paths and group clips,
  alpha resources are shared by primitive, path, and group consumers, and leaf
  ordinals agree between collection and recursive emission. No jointly
  incorrect feature interaction was found.
- `duplication`: no competing path geometry, alpha registry, or recursive walk
  helper was introduced.
- `layering`: `oxml-pdf` depends only on `oxml-layout`, `oxml-media`, and its
  external implementation crates at `crates/oxml-pdf/Cargo.toml:15`. It has no
  `rdocx-*` or `rpptx-*` edge.
- `harness`: no hash or golden manifest changed in S09. Both exact integrated
  comparisons passed.
- `gate`: the feature tests, exact raster comparison, injected-pixel rejection,
  workspace gate, package dry-run, archive-size check, and supply-chain check
  cover the sprint definition of done.
- `deps`: no manifest or lockfile changed, so S09 introduced no dependency.
- `surface`: all new renderer state and helpers are private. No public API was
  added.
- `publication`: `oxml-pdf` remains at version 0.0.0 with `publish = false` at
  `crates/oxml-pdf/Cargo.toml:4`. The other staged `oxml-*` crates retain the
  same boundary, and no crate was published.
- `ledgers`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree that
  F-040, F-041, F-042, and F-044 are complete in S09.
