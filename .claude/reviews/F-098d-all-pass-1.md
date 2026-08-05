# F-098d, all, pass 1

**Reviewed**: working diff against `26f570627fb2a8600b9c976e297f3e66831eea46`, 3 files, 274 insertions and 7 deletions. The scope is 5 insertions and 5 deletions in `.claude/plans/F-098d-design.md`, 262 insertions and 2 deletions in `crates/rpptx-render/src/text.rs`, and 7 insertions in `docs/hld/08-rendering-spec.md`.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No correctness findings. Top, centre, bottom, justified, and distributed
offsets match the approved spare-height policies, including negative overflow
and the single-line policy.

No contract findings. Horizontal alignment is applied before vertical
translation, line decorations move with their glyph runs, and production-path
coverage proves path-before-text order without clipping.

No panic findings. The only new slice bounds come from element vector lengths
recorded immediately before and after emission, and every divisor is guarded by
the applicable line-count condition.

No OOXML findings. This diff changes no parser, serializer, namespace handling,
schema child ordering, or preservation behavior.

No test findings. The four approved tests distinguish ordinary anchor offsets,
negative overflow, justified and distributed allocation, and the required
bottom-centre inset-box baseline. Each test uses deterministic fonts, and the
baseline gate computes the expected placement independently of the anchor
implementation.

No structure findings. The implementation stays in the approved private module
and introduces no trait, generic parameter, wrapper, feature flag, crate,
module, or source file.
