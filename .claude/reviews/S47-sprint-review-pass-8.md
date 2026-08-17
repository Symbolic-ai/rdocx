# S47 sprint review, pass 8

**Reviewed**: `sprint/s47` at `29f406b` against `d625bda4`, 50 files,
7,050 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 8 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 3 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, retained raw run properties can still follow the schema-final change

`crates/rdocx-oxml/src/properties.rs:993`
`crates/rdocx-oxml/src/properties.rs:1020`
`crates/rdocx-oxml/src/properties.rs:1192`
`crates/rdocx-oxml/src/properties.rs:1195`
`crates/rdocx-oxml/src/revision.rs:942`

Raw content encountered after a modeled `w:rPrChange` remains pending until
the parser assigns it the end slot. Both serializer paths then emit the
modeled change before the end-slot raw content. The strengthened pass-7
regression explicitly asserts `w:afterChange` after `w:rPrChange`, so it locks
in the opposite of the pass-7 requirement that `w:rPrChange` remain the final
child.

This can still produce schema-invalid run properties and leaves pass-7 B1
open. The fix must serialize every retained child before the schema-final
change, including a child read after it from hostile producer XML, and assert
that no child follows `w:rPrChange` in the emitted `w:rPr`.

### B2, duplicate and raw-only properties outlive their occurrence anchors

`crates/rdocx-oxml/src/properties.rs:175`
`crates/rdocx-oxml/src/properties.rs:767`
`crates/rdocx-oxml/src/properties.rs:976`
`crates/rdocx-oxml/src/properties.rs:1020`
`crates/rdocx-oxml/src/properties.rs:1103`
`crates/rdocx-oxml/src/properties.rs:1412`

The sidecar records an occurrence for every parsed modeled property, but
scalar fields retain only the last occurrence and the canonical writer emits
only one. In `<w:b w:val="0"/><x:raw/><w:b/>`, the raw child is anchored before
bold occurrence 1. The generated output has only occurrence 0, so the merge
delays the raw child until after the surviving bold even though the surviving
value came from the property after it.

The same representation assigns every raw-only `w:rPr` child to the end slot
because raw content is flushed only when a later modeled child appears. A
valid non-self-closing property such as `w:rStyle` enters the generic start
element branch. If the public run API then materializes bold, the writer puts
bold before the raw style even though `w:rStyle` is the first schema slot.

This leaves the duplicate, absent-slot, and raw-only cases requested by the
pass-8 audit unproved and can reorder or invalidate retained producer XML
after an established formatting mutation. The fix must retain an anchor that
survives canonical occurrence collapse and must classify raw Word property
names into their schema slots even when no modeled neighbour is present.

### B3, display replacement inserts text after collapsed trailing raw children

`crates/rdocx/src/content_control.rs:393`
`crates/rdocx/src/content_control.rs:396`
`crates/rdocx/src/content_control.rs:408`
`crates/rdocx/src/content_control.rs:426`
`crates/rdocx-oxml/src/text.rs:239`

Content-control display replacement removes non-text run content, remaps its
raw boundaries as a pure deletion, and only then inserts replacement text at
content index zero when the run had no ordinary text. A run containing
`raw-before`, a tab or drawing, and `raw-after` therefore collapses both raw
children to boundary zero before the new text is inserted. It serializes as
`raw-before`, `raw-after`, replacement text instead of retaining the raw
children around the replacement display location.

The pass-7 remediation covered `set_text`, `add_text`, formatting, and direct
and content-control comment removal, but its regression does not exercise
display replacement without an existing text node. The fix must remap the
insertion as well as the deletion and add a facade regression through
`set_content_control_value_by_tag` or the equivalent public operation.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish the complete gate because the sprint plan assigns the
end-of-milestone gate to S48. For the S47 contribution, all 206 `rdocx-oxml`
tests, all 52 `rdocx` regression tests, all 87 `rdocx` integration tests, and
all 49 hash-harness entries pass at the reviewed SHA. The focused pass-7 tests
establish their direct run mutation, comment removal, namespace repair, and
single-occurrence property fixtures. B1 through B3 identify schema order and
mutation cases outside that evidence, so the tracked-change preservation
contribution remains blocked.

## Not found

- `prior review findings`: pass-1 B1, pass-2 B1 and B2, pass-3 B1, pass-4 B1
  through B3, pass-5 B1, and pass-6 B1 and B2 remain fixed for their cited
  direct cases. Pass-7 B3 and B4 are fixed. Pass-7 B1 and B2 remain open only
  through B1 through B3 above.
- `namespace repair`: parsed qualified names and exact declarations now handle
  empty roots, nested declarations, `xmlns:w2`, quoted `>`, and literal `w:`
  values or text without a separate defect. Parse failure leaves the raw bytes
  untouched. Zero additional namespace findings.
- `revision reporting and resolution`: modeled revisions report once in the
  tested document order, malformed owners stay opaque, selectors retain exact
  counts, and selected nesting resolves inside out with atomic commit. Zero
  additional reporting or resolver findings.
- `numbering sole source`: numbering clears both the raw property projection
  and its position sidecar before overlay serialization. The no-duplication
  and namespace regressions pass. Zero numbering findings.
- `surface and docs`: HLD 03, HLD 10, the F-149 plan, and AS_BUILT now identify
  `RunContent::DeletedText` and the required fields on `CT_R`, `CT_P`,
  `HyperlinkSpan`, `CT_PPr`, `CT_RPr`, `CT_SectPr`, `CT_TblPr`, and `CT_TrPr`
  as the intentional low-level 0.8.0 boundary. The `rdocx::Document` facade is
  additive and Python, WASM, and CLI surfaces remain unchanged. Zero surface
  or documentation findings.
- `interaction`: outside B1 through B3, no conflict was found between the
  F-149 projection and F-150 resolver.
- `duplication`: no duplicate sprint helper or second public revision model
  was introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `deps`: no dependency was added.
- `harness`: the independent check reports all 49 entries unchanged, matching
  both S47 AS_BUILT entries.
- `oracle`: the normalized-body regression remains pinned to Microsoft Word
  16.104 build 16.104.25121423 and compares normalized typed body structure.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on
  both completed stories, estimates, actuals, HLD files, and unchanged harness
  evidence.
