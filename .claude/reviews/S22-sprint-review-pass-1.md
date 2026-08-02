# S22 sprint review, pass 1

**Reviewed**: `sprint/s22` against
`de614a1ab280e4deb783bb938343692190f99664`, 29 files, 22,473 changed lines,
crates: `oxml-drawing`, `rpptx-layout`, `rpptx-render`
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

S22 does not complete M10, so this gate is not yet due and is not claimed. The
four S22 slice gates hold. The HLD records the permitted Ecma source and licence
basis at `docs/hld/13-risks-and-open-questions.md:5`. The generator proves
byte-identical output and corpus coverage at `tools/gen-presets/generate.py:236`.
Known and unknown preset behavior is exercised at
`crates/rpptx-layout/src/context.rs:2407`, and the unknown fallback retains text
and a named diagnostic at `crates/rpptx-layout/src/context.rs:2169`. Scoped
slide, layout, and master relationship lookup is exercised at
`crates/rpptx-render/src/lib.rs:173`. The integrated full workspace gate and all
28 unchanged hashes passed before this review.

## Not found

- Interaction: the generated preset table feeds the shared guide evaluator,
  and the renderer assembly seam consumes the same frozen resolver output
  without duplicating inheritance or geometry logic.
- Duplication: there is one preset lookup, one guide evaluator, one scoped
  media resolver, and one `RenderInput` boundary.
- Layering: `rpptx-render` depends inward on the model and layout crates. No
  `oxml-*` crate gained an `rpptx-*` dependency.
- Harness: all four AS_BUILT entries record the observed unchanged 28-entry
  result, and no baseline file changed.
- Gate: every S22 story has a named non-vacuous gate. The corpus exercises
  2,141 preset uses in the generator scan and 921 preset inputs in resolution.
- Docs: the HLD records both the provenance decision and the split between raw
  `SlideBundle` assembly and owned `RenderInput` consumption.
- Deps: the new normal dependencies are workspace crates with concrete fields
  or functions in `rpptx-render`. No third-party dependency was added.
- Surface: the new public types are the F-091 geometry model and F-092 renderer
  assembly contract requested by the approved stories. All PowerPoint crates
  remain version 0.0.0 with publication disabled.
