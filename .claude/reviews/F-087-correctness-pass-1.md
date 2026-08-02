# F-087, correctness, pass 1

**Reviewed**: working diff, 6 files, 1,316 changed lines
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, custom geometry treats path-local coordinates as EMU

`crates/rpptx-layout/src/context.rs:672`

`CT_CustomGeometry2D::evaluate` returns coordinates in each path's declared
coordinate space. The conversion loses each path's width and height, then
divides every coordinate by 12,700 as though it were an EMU. A common 21,600
wide custom path inside a 127,000 EMU shape therefore resolves to about 1.7
points instead of the shape's 10 point width. The method needs the shape bounds
and the source path dimensions, or it must retain a diagnosed bounds fallback.

### D2, the corpus gates do not read the corpus

`crates/rpptx-layout/src/context.rs:1967`

`all_corpus_slides_resolve_without_panics` iterates three in-memory XML strings.
It never opens the pinned 50-deck corpus and passes when that corpus is absent.
The same limitation affects the named backlog gate, so malformed or unsupported
real slides cannot fail either promised corpus test.

### D3, nested group transforms disappear before the frozen contract

`crates/rpptx-layout/src/context.rs:1222`

The flattener recurses into a group without carrying its transform, and
`ResolvedShape` has only leaf bounds, rotation, and flips at
`crates/rpptx-layout/src/lib.rs:30`. A translated, scaled, or rotated group thus
emits children in the right order but at untransformed positions. Because this
story freezes the renderer boundary, the later renderer has no source value
from which to recover the missing affine transform.

### D4, table text body properties are replaced with hardcoded defaults

`crates/rpptx-layout/src/context.rs:1125`

The table path reads insets and wrap, but always emits top anchoring, horizontal
text, and no autofit. A cell with centre anchoring, vertical text, or stored
normal autofit resolves to the wrong contract even though each property is
already typed in the source model.

### D5, the frozen paragraph and bullet types omit renderer inputs

`crates/rpptx-layout/src/lib.rs:130`

`ResolvedParagraph` has no line spacing, space before, or space after fields,
and `ResolvedBullet` has no bullet-size field. Those modelled values affect
fixed-box slide text layout, but the source values are dropped before the
renderer boundary. F-098 cannot implement the HLD text algorithm from this
frozen type set.

### D6, gradients are marked concrete after modelled geometry is discarded

`crates/rpptx-layout/src/context.rs:519`

Every path gradient becomes the same centred radial paint, regardless of its
shape and fill rectangle. Linear gradients also discard scaling, flip,
rotation-with-shape, and tile rectangle values. The result carries no
`unsupported` marker or diagnostic, so the renderer is told that an
approximation is a resolved paint. Unsupported modelled forms must retain a
diagnosed fallback unless the conversion preserves their semantics.

## Smells

None.

## Nitpicks

None.

## Not found

No panics on untrusted input, indexing defects, schema-order changes, parser or
serializer changes, source-model types in the public contract, reverse
dependency edges, new traits, new generic parameters, new feature flags, or
new source modules were found. The large `context.rs` diff remains locally
partitioned into shape, paint, geometry, text, and table helpers. Its size alone
does not justify violating the approved no-new-module constraint.
