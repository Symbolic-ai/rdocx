# F-X015, correctness, pass 1

**Reviewed**: the uncommitted working tree. `rdocx-oxml/src/drawing.rs` for the
model, parser and serialiser, `rdocx-layout/src/block.rs` and `engine.rs` for
carrying the fields into layout.
**Verdict**: 0 defects, 0 smells, 2 nitpicks

## Defects

None outstanding.

### D1 found and fixed during the pass, the serialiser moved output
`crates/rdocx-oxml/src/drawing.rs:706`

The story is defined as model-only, and its proof is a flat harness. The first
implementation wrote `distT`, `distB`, `distL` and `distR` unconditionally,
which changed `report:word/document.xml`, because the sample generators build
the report's background anchor programmatically rather than parsing one. The
harness caught it on the first run.

A zero distance is the default and an absent attribute means the same thing, so
the four are now written only when non-zero. Semantically identical output,
harness back to 28 of 28, and the story is model-only again. The plan has been
corrected: "only a programmatically built anchor" was true and still missed that
the samples are exactly that.

Worth stating plainly, because it is the one thing that went right here by
process rather than by care: the design predicted no delta, the prediction was
wrong, and the gate that exists for precisely this caught it before the commit.

## Smells

None.

## Nitpicks

- `crates/rdocx-oxml/src/drawing.rs:706`, the four `to_string` bindings are
  computed before the `if` that may not use them. Harmless, and hoisting them
  keeps each borrow alive long enough for `push_attribute`, which is why they
  sit there.
- `crates/rdocx-oxml/src/drawing.rs`, `AnchorAlignH::parse` and
  `AnchorAlignV::parse` are byte-for-byte parallel. Merging them would need a
  trait or a macro for five string literals each, which costs more than it
  saves under the structural rules in `CLAUDE.md`.

## Not found

Checked and produced nothing:

- **correctness**. `wrap_type_of` covers all five wrapping elements and is used
  for both the empty and the expanded spelling, with the expanded one skipping
  its subtree so an unmodelled `wrapPolygon` cannot derail the walk. An unknown
  alignment reads as `None`, which falls back to the offset rather than
  inventing a position.
- **panics**. Distance attributes parse with `unwrap_or(0)`. No indexing or
  slicing added.
- **ooxml**. Prefix-tolerant on read through `matches_local_name`, proven by the
  wrap and alignment tests. Fixed `wp:` prefix on write. The wrap element keeps
  the sequence position `wrapNone` held. `a_parsed_anchor_re_emits_its_original_bytes`
  proves the `raw_xml` capture path still returns an unmodelled subtree and an
  unmodelled attribute byte for byte, so the preservation contract is intact.
  An anchor writes an alignment or an offset, never both, which is what the
  schema allows.
- **structure**. Two new enums and five new struct fields, all with a named
  consumer: F-X016 reads every one. No new trait, generic, module or file.
- **contract**. Matches the plan, with the serialiser correction recorded in the
  plan itself.
- **tests**. The wrap and round-trip tests fail against reverted wrap parsing.
  The alignment and round-trip tests fail against reverted distance and
  alignment reads. Both reverts were run.
