# S47 sprint review, pass 7

**Reviewed**: `sprint/s47` at `4e86fd7` against `d625bda4`, 48 files,
6,183 changed lines, crates: `rdocx-oxml`, `rdocx`, `rdocx-layout`, and
`rdocx-html`
**Review-bound extension**: Pass 7 continues under the explicit authorization
recorded in pass 4 on 2026-08-17 for as many passes as required to reach a
clean verdict.
**Verdict**: 4 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, retained run-property children still lose their original order

`crates/rdocx-oxml/src/properties.rs:786`
`crates/rdocx-oxml/src/properties.rs:808`
`crates/rdocx-oxml/src/properties.rs:963`
`crates/rdocx-oxml/src/properties.rs:969`

The pass-6 remediation now captures every foreign or unmodelled child of
`w:rPr`, but puts all of them into the flat `revision_xml` list. Serialization
writes modeled properties first, then contextual revisions, then the
schema-final `w:rPrChange`, and only then the complete raw list. A foreign
child before `w:b`, or between `w:b` and `w:i`, therefore moves to the end. A
raw child that preceded `w:rPrChange` moves after that schema-final child.

The pass-6 regression covers a foreign property that was already after its
only typed sibling, so it proves retention and expanded identity but not this
ordering case. The fix must retain raw run-property children at their original
typed-property boundaries and keep `w:rPrChange` final, with before, between,
and after regressions under a shadowed Word prefix.

### B2, run raw-child positions are not maintained by existing mutations

`crates/rdocx-oxml/src/text.rs:386`
`crates/rdocx-oxml/src/text.rs:468`
`crates/rdocx-oxml/src/text.rs:492`
`crates/rdocx/src/run.rs:83`
`crates/rdocx/src/run.rs:88`
`crates/rdocx/src/run.rs:395`
`crates/rdocx-oxml/src/text.rs:708`
`crates/rdocx/src/comments.rs:976`

`extra_xml_positions` counts run properties and typed content, but the public
run API replaces and appends content and can materialize `w:rPr` without
remapping those positions. Comment removal also retains away a typed comment
reference in direct runs and content-control runs without remapping them. A raw
child anchored after a removed or replaced second typed child can consequently
have a boundary the writer never visits and disappear. Adding formatting to a
parsed run with a raw child at boundary zero emits that raw child before the
new `w:rPr`, violating the rule that run properties are first.

This is an interaction between the pass-6 ordering model and established
native, binding, content-control, and comment mutations. The fix must make
every content or property mutation preserve and remap run-local raw boundaries,
then cover `set_text`, `add_text`, formatting materialization, direct comment
removal, and content-control comment removal with raw children before, between,
and after typed content.

### B3, the sprint contains undeclared breaking changes to published Rust types

`crates/rdocx-oxml/src/text.rs:54`
`crates/rdocx-oxml/src/text.rs:57`
`crates/rdocx-oxml/src/text.rs:161`
`crates/rdocx-oxml/src/text.rs:168`
`crates/rdocx-oxml/src/text.rs:540`
`crates/rdocx-oxml/src/text.rs:555`
`.claude/plans/F-149-design.md:164`
`Cargo.toml:34`

`RunContent` is a public exhaustive enum and gained `DeletedText`, so a
downstream exhaustive match stops compiling. `CT_R`, `CT_P`, `HyperlinkSpan`,
`CT_PPr`, `CT_RPr`, `CT_SectPr`, and table property structs are public and
gained required fields, so downstream full struct literals also stop
compiling. `#[doc(hidden)]` does not remove a public field from the Rust API.

The design plan classifies the low-level changes as additive, the S47 HLD and
AS_BUILT updates record only additive native facade APIs, and the published
family remains at 0.7.0. This repository has previously treated the same enum
and struct-literal changes as an explicit breaking pre-1.0 boundary. The fix
must either make the low-level representation source-compatible or state and
record the exact breaking surface and its next-release version boundary. The
eight `rdocx::Document` resolution methods themselves remain additive.

### B4, raw namespace repair uses lexical substring tests instead of XML names

`crates/rdocx-oxml/src/text.rs:1719`
`crates/rdocx-oxml/src/text.rs:1723`
`crates/rdocx-oxml/src/text.rs:1728`
`crates/rdocx-oxml/src/text.rs:1740`

The shadowed-prefix writer decides that raw XML uses `w` by searching every
byte for `w:`, decides that the root declares `w` by searching the opening
bytes for `xmlns:w`, and finds the opening tag end at the first `>` byte. A
root carrying `xmlns:w2` is therefore mistaken for one declaring `w`. A nested
`w:` name then inherits the reconstructed run's Word binding instead of the
foreign binding it had in the input. Text or an attribute value containing
`w:` causes an unnecessary declaration, while a quoted attribute value
containing `>` can make the insertion point fall inside that value.

The fix must inspect parsed qualified names and exact namespace declarations,
not byte substrings, and must insert on the parsed root start element. Add
regressions for `xmlns:w2`, nested `w:` use, literal `w:` text or values, and a
quoted `>` under the locally shadowed hyperlink scope. Each raw subtree must
remain well formed, retain its expanded names, and appear once.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M14 gate is: "a document carrying tracked changes, comments, content
controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API."

S47 does not establish that gate because the sprint plan assigns the complete
end-of-milestone gate to S48. For the S47 contribution, all 206 `rdocx-oxml`
tests, all 50 `rdocx` regression tests, all 87 `rdocx` integration tests, and
all 49 hash-harness entries pass at the reviewed SHA. The strengthened pass-6
test proves expanded-name parsing and ordered raw run children for its exact
load and save fixture. B1, B2, and B4 show that property order, mutations, and
additional namespace scopes remain outside that evidence. B3 leaves the
published compatibility impact inaccurately recorded.

## Not found

- `prior review findings`: pass-1 B1, pass-2 B1 and B2, pass-3 B1, pass-4 B1
  through B3, and pass-5 B1 remain fixed for their cited cases. The direct
  pass-6 foreign same-local parsing cases now remain foreign and run-local raw
  children retain the tested before, between, and after order.
- `revision reporting and resolution`: modeled revisions report once in the
  tested document order, malformed owners remain opaque, and selected nested
  revisions resolve inside out with atomic commit. No separate resolver
  finding was found.
- `numbering sole source`: numbering clears the duplicate typed raw projection
  while its property overlay remains the preservation source.
  `numbering_preservation_does_not_duplicate_typed_changes` and the numbering
  namespace and overlay regressions pass.
- `comment and hyperlink boundaries`: the pass-4 paragraph and hyperlink
  boundary remapping cases remain fixed. B2 is limited to the newly introduced
  run-local raw boundary sidecar.
- `duplication`: no duplicate sprint helper or second public revision model was
  introduced.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: the independent check reports all 49 entries unchanged, matching
  both S47 AS_BUILT entries.
- `deps`: no dependency was added.
- `oracle`: the normalized-body regression records Microsoft Word 16.104
  build 16.104.25121423 and compares normalized typed body structure rather
  than producer bytes.
- `delivery`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, and AS_BUILT agree on
  the two completed stories, their estimates and actuals, HLD files, and
  unchanged harness evidence. The compatibility and preservation claims are
  blocked by B1 through B4 rather than by ledger totals.
