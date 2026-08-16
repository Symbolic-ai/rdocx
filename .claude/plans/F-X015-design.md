# F-X015, Anchored drawing wrap and alignment model

**Status**: completed
**Sprint**: S41
**Size**: M
**Depends on**: none

## Problem

`WrapType` in `crates/rdocx-oxml/src/drawing.rs:105` has one variant, `None`.
`CT_Anchor::from_xml` recognises `wrapNone` and ignores every other wrap
element, and `CT_Anchor::to_xml` writes `wrapNone` unconditionally at line 575.
The `wrap` field is set to `WrapType::None` at both construction sites and read
nowhere, so it is parsed-but-dead: the model records a wrap mode it never
learns and never uses.

Three further pieces of the anchor are dropped on read:

- `distT`, `distB`, `distL` and `distR`, the space a wrapped drawing keeps
  between itself and the text around it.
- The `wp:align` child of `positionH` and `positionV`. An anchor positions
  itself either by an offset or by an alignment, and only the offset is read.
  An alignment-positioned drawing therefore lands at offset zero.

None of this loses data on round trip, because `CT_Drawing::from_xml` captures
the whole `wp:anchor` into `raw_xml` and `to_xml` re-emits those bytes verbatim.
The loss is to layout, which cannot see what it needs to wrap text.

This story adds the model surface and changes no placement or rendering. That is
deliberate: it keeps the hash harness flat, which is what proves the story is
model-only, and leaves F-X016 owning the entire rendering delta.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put", for the `wp:` anchor code
  being Word-only and staying in `rdocx-oxml` rather than migrating.
- `docs/hld/05-drawingml-model.md`, "Do not touch the Word path", which is what
  keeps this work out of `oxml-drawing`.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the round-trip
  category, and "The hash harness" for the labelled-delta rule.
- `docs/hld/14-development-backlog.md`, "F-X015, Anchored drawing wrap and
  alignment model".

## Approach

### `WrapType` gains the modes documents actually use

```rust
pub enum WrapType {
    None,
    Square,
    TopAndBottom,
    Tight,
    Through,
}
```

All four wrapping elements are parsed distinctly rather than collapsed. `Tight`
and `Through` wrap to the drawing's outline rather than its frame, which F-X016
will approximate as `Square`, but that approximation belongs in the renderer.
Collapsing them here would throw away information the model cannot recover.

### `CT_Anchor` carries the distances and the alignments

```rust
pub dist_t: Emu,
pub dist_b: Emu,
pub dist_l: Emu,
pub dist_r: Emu,
pub pos_h_align: Option<AnchorAlignH>,
pub pos_v_align: Option<AnchorAlignV>,
```

with

```rust
pub enum AnchorAlignH { Left, Center, Right, Inside, Outside }
pub enum AnchorAlignV { Top, Center, Bottom, Inside, Outside }
```

Read prefix-tolerantly through the existing `matches_local_name`. `to_xml`
writes the four distances, the alignment when present, and the wrap element
matching `self.wrap` rather than an unconditional `wrapNone`. That path only
runs for a programmatically built anchor, since a parsed one re-emits its
captured bytes.

**Corrected during implementation.** "Only a programmatically built anchor"
turned out to include the sample generators, which build the `report` sample's
background anchor. Writing `distT="0"` and its siblings changed that sample's
`document.xml` and broke the harness, which is precisely the signal the harness
exists to give. A zero distance is the default and an absent attribute means the
same thing, so the four are now written only when non-zero. Semantically
identical, and the story stays model-only.

An unknown alignment string parses as `None`, meaning "no alignment given", so
the offset is used. That is the tolerant read the domain rules ask for and it
degrades to today's behaviour.

### `AnchoredDrawing` carries them into layout

```rust
pub wrap: WrapType,
pub dist_top: f64,
pub dist_bottom: f64,
pub dist_left: f64,
pub dist_right: f64,
pub align_h: Option<AnchorAlignH>,
pub align_v: Option<AnchorAlignV>,
```

Populated by `collect_anchored` in the engine, in points. The paginator does not
read them, so placement is unchanged.

## Rejected alternatives

- **Collapse `Tight` and `Through` into `Square` at parse time.** Loses which
  the document asked for, and the model cannot recover it. Approximating is the
  renderer's job.
- **Add the fields to `AnchoredDrawing` only, not to `CT_Anchor`.** The
  serialiser would keep writing `wrapNone` for a built anchor whose wrap says
  otherwise, so the model would contradict its own output.
- **Do this inside F-X016.** One commit mixing a model addition with a layout
  change leaves a harness delta that cannot be attributed to a cause.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `an_anchor_round_trips_its_wrap_distances_and_alignments` | A built anchor carrying each wrap mode, the four distances and both alignments survives a serialise and parse cycle unchanged |
| round-trip | `a_parsed_anchor_re_emits_its_original_bytes` | A parsed anchor still round trips verbatim through `raw_xml`, so this story adds no serialisation drift |
| unit | `every_wrap_element_parses_to_its_own_mode` | `wrapNone`, `wrapSquare`, `wrapTopAndBottom`, `wrapTight` and `wrapThrough` each parse to a distinct variant, in both empty and expanded element spellings |
| unit | `anchor_alignments_and_distances_are_read` | `distT/B/L/R` and a `wp:align` in each axis are read, through a foreign namespace prefix |
| unit | `an_unknown_alignment_reads_as_no_alignment` | An unrecognised `wp:align` value leaves the alignment `None` so the offset is used |

**Test gate**, from the backlog: the round-trip pair.

## HLD impact

None. The story adds model surface for a construct the spec set already
describes, and changes no documented behaviour. F-X016 carries the HLD update
for wrapping, since that is where the behaviour appears.

## Risk routing

Matched row: **Any parser or serialiser**.

- Prefix-tolerant on read through `matches_local_name`, fixed `wp:` prefix on
  write.
- `xsd:sequence` child order on write: the wrap element follows the position and
  extent elements, which is where `wrapNone` already sat.
- A round-trip test proving a parsed anchor still re-emits its captured bytes
  byte for byte, so the `raw_xml` preservation path is not disturbed.

The layout row does **not** match, and that is the point of the story. No
pagination, line breaking or shaping code is touched.

## Hash harness

**Unchanged, 28 of 28**, but not on the first attempt. The serialiser initially
wrote all four distances unconditionally, which changed `report:word/document.xml`
because the sample generators build an anchor programmatically. The harness
caught it, the cause was identified as four zero-valued attributes that
previously did not appear, and the serialiser now omits a zero distance. The
final state moves no output, which is what proves the story is model-only.

## Implementation checklist

- [x] `WrapType` variants and the two alignment enums
- [x] `CT_Anchor` fields, parsed prefix-tolerantly
- [x] `to_xml` writes the distances, alignments and the real wrap element
- [x] `AnchoredDrawing` fields, populated in points by the engine
- [x] Tests, including the parsed-anchor byte-for-byte round trip
- [x] Confirm the harness is unchanged, which is the story's proof
- [x] `/microscope F-X015 --working`
- [x] `/verify`

## Open questions

None.
