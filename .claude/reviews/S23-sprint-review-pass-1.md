# S23 sprint review, pass 1

**Reviewed**: `sprint/s23` against
`8b8313983bed13bda3b7e38f2c820d84a4fb3d53`, 30 files, 4,606 changed lines,
crates: `oxml-pdf`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M10 end gate at `docs/hld/14-development-backlog.md:707` is: "the SSIM
harness meets its target across the corpus."

S23 does not complete M10, so this gate is not yet due and is not claimed. The
five S23 slice gates hold. Shape fill, gradient, and outline pixels are sampled
at `crates/rpptx-render/src/lib.rs:1296`. Independently computed rotated corners
are checked at `crates/rpptx-render/src/lib.rs:1117`. A triangular tail produces
an additional closed filled path and raster evidence at
`crates/rpptx-render/src/lib.rs:1495`. Crop-region exclusion is sampled at
`crates/rpptx-render/src/lib.rs:1637`. An inherited master gradient is resolved
and rasterised at `crates/rpptx-render/src/lib.rs:2171`. The integrated full
workspace gate and all 28 unchanged hashes passed before this review.

## Not found

- Interaction: shape geometry, transforms, arrowheads, picture content, and
  backgrounds meet at one ordered page-frame lowering path at
  `crates/rpptx-render/src/lib.rs:189`. Picture content precedes its outline,
  endpoint paths follow the stroked geometry, and the page background remains
  outside the ordinary element list.
- Duplication: shape placement has one transform composer at
  `crates/rpptx-render/src/lib.rs:967`. Picture crop, clipping, tile placement,
  and endpoint geometry each have one lowering path. Background style lookup
  reuses the existing fill-style helper.
- Layering: the only new manifest edges are test-only `oxml-pdf` and
  `tiny-skia` consumers at `crates/rpptx-render/Cargo.toml:21`. No `oxml-*`
  crate gained an `rpptx-*` dependency.
- Harness: all five AS_BUILT entries record the observed unchanged 28-entry
  result, beginning at `docs/sprints/AS_BUILT.md:2790`, and no baseline file
  changed.
- Gate: every S23 story has a named non-vacuous gate with the evidence above.
  The raster gates use generated fixtures at 72 DPI and no system-font
  baseline.
- Docs: the preserving background projection is current at
  `docs/hld/06-presentationml-model.md:134`, the neutral resolver amendments at
  `docs/hld/07-inheritance-and-resolution.md:24`, and picture and endpoint
  lowering at `docs/hld/08-rendering-spec.md:126` and
  `docs/hld/08-rendering-spec.md:399`.
- Deps: no third-party normal dependency was added. The two dev dependencies
  have concrete deterministic raster-test consumers.
- Surface: the public line-end, picture-placement, background-projection, and
  layout entry points are the exact surfaces requested by F-093 through F-097.
  All PowerPoint crates remain version 0.0.0 with publication disabled, as
  shown for the renderer at `crates/rpptx-render/Cargo.toml:2`.
