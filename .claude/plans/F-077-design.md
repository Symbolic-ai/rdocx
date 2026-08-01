# F-077, Notes slides and notes master

**Status**: approved
**Sprint**: S18
**Size**: M
**Depends on**: F-069

## Problem

`rpptx-oxml` models slides, layouts, and masters in
`crates/rpptx-oxml/src/slide_parts.rs:58`, but it has no roots for notes slides
or notes masters. The shared shape model also exposes only placeholder metadata
at `crates/rpptx-oxml/src/shape_tree.rs:50`, leaving `p:txBody` opaque. Callers
therefore cannot extract speaker notes or round-trip notes parts through typed
part boundaries.

A raw scan for every `a:t` would include slide-number fields and notes-master
prompt text. Speaker-note extraction needs the effective body placeholder and
the existing DrawingML text model as its single source of truth.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "PresentationML scope".
- `docs/hld/04-opc-and-packaging.md`, "Preservation discipline".
- `docs/hld/05-drawingml-model.md`, "Text" and "Preservation".
- `docs/hld/06-presentationml-model.md`, "Parts" and "The shape tree".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-077, Notes slides and notes master".

## Approach

Add `crates/rpptx-oxml/src/notes_parts.rs` with schema-shaped roots:

```rust
pub struct CT_NotesSlide {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map_override: Option<CT_ColorMapOverride>,
    // private raw attributes and ordered raw children
}

impl CT_NotesSlide {
    pub fn from_xml(xml: &[u8]) -> Result<Self>;
    pub fn to_xml(&self) -> Result<Vec<u8>>;
    pub fn notes_text(&self) -> String;
}

pub struct CT_NotesMaster {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map: ColorMap,
    pub notes_style: CT_TextListStyle,
    // private raw attributes and ordered raw children
}
```

The notes-slide sequence is `p:cSld`, optional `p:clrMapOvr`, then optional
`p:extLst`. The notes-master sequence is `p:cSld`, required `p:clrMap`, optional
raw `p:hf`, required typed `p:notesStyle`, then optional `p:extLst`. Both roots
reuse the existing common-slide, colour-map, and ordered-preservation machinery,
accept namespace aliases on read, and write fixed prefixes in schema order.

Extend `CT_Shape` with `pub text_body: Option<CT_TextBody>` for `p:txBody`.
Reuse the existing DrawingML text parser and add an internal wrapper-aware
writer so shape XML emits `p:txBody`, while table cells continue to emit
`a:txBody`. Add `CT_TextBody::plain_text()` to concatenate run and field text in
document order, convert explicit breaks to newlines, and separate paragraphs
with newlines.

`CT_NotesSlide::notes_text()` walks shapes in z-order and includes only text
from placeholders whose effective type is `body`. It excludes slide image,
slide number, date, footer, and header placeholders. It also excludes notes
master prompt text. Unsupported notes children remain captured at their schema
boundaries.

F-077 integrates after F-075 and F-076 because all three update shape-tree
dispatch. No publication or version change is part of the story.

## Rejected alternatives

- Scan raw XML for `a:t`. That returns fields and master prompts that are not
  speaker notes and duplicates the typed text parser.
- Keep `p:txBody` opaque and cache an extracted string. That creates two sources
  of truth after edits.
- Put notes roots into `slide_parts.rs`. That file is already large, while
  notes slides and notes masters form one cohesive second part family.
- Create another integration-test file. Repository guidance requires modules in
  the existing test binary to avoid another binary link.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `text_body_plain_text_preserves_run_field_break_and_paragraph_order` | Runs, fields, explicit breaks, and paragraphs produce the specified newline-separated text |
| unit | `notes_slide_extracts_only_body_placeholder_text` | Slide image, slide number, footer, and master prompt text are excluded from speaker notes |
| unit | `notes_parts_read_any_prefix_write_fixed_prefixes_and_schema_order` | Alias prefixes parse and both roots write their required and optional children in schema order |
| preservation | `notes_parts_preserve_unmodelled_children_in_their_schema_slots` | Producer attributes and unsupported children remain exact at their original boundaries |
| round-trip | `every_corpus_notes_slide_and_master_round_trips_structurally` | Every content-type-discovered notes part serialises and reparses equally, with nonempty body notes extracted |
| integration | `corpus_notes_relationships_are_complete` | Notes slides, notes masters, themes, and source slides have the required relationship cardinalities |

The test gate is: notes text extracts, and a deck with notes round-trips.

The pinned corpus currently supplies 210 notes slides, 24 notes masters, and 72
nonempty speaker-note bodies. The gate asserts positive coverage without making
producer counts the public API contract.

## HLD impact

- `docs/hld/06-presentationml-model.md`, define notes-root child sequences,
  relationship cardinalities, and speaker-note extraction as effective body
  placeholder text only.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Check both root sequences,
  prefix-tolerant reads, fixed-prefix writes, and byte preservation of every
  unsupported subtree.
- Crate dependency graph and a new family `use`. Keep the existing dependency
  direction from `rpptx-oxml` to `oxml-drawing` and confirm it with
  `cargo tree -p rpptx-oxml --edges normal`.
- A new module or file. Read the structural rules in `CLAUDE.md` and obtain
  explicit approval before adding `crates/rpptx-oxml/src/notes_parts.rs`.

The consolidated sprint gate adds `cargo test -p oxml-drawing`,
`cargo test -p rpptx-oxml`, the two required notes corpus tests, and the
dependency-tree check.

## Hash harness

Expected to be unchanged. Notes support remains inside unpublished PowerPoint
development crates and does not modify the released Word path.

## Implementation checklist

- [ ] Add and export notes-slide and notes-master roots.
- [ ] Enforce their required child sequences and preserve optional content.
- [ ] Type `p:txBody` on shapes using the existing DrawingML text model.
- [ ] Add plain-text extraction for runs, fields, breaks, and paragraphs.
- [ ] Extract speaker notes only from effective body placeholders.
- [ ] Add focused schema, extraction, relationship, and preservation tests.
- [ ] Add the required pinned-corpus notes round-trip gate.
- [ ] Update the approved HLD impact file.
- [ ] Confirm every PowerPoint development crate remains version 0.0.0 and unpublished.
- [ ] Confirm all deterministic hashes remain unchanged.

## Open questions

None. The user approved `crates/rpptx-oxml/src/notes_parts.rs`, typed
`p:txBody` on shapes, and speaker-note extraction from effective body
placeholders only.
