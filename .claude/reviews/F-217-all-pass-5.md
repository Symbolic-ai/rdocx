# F-217, all, pass 5

**Reviewed**: uncommitted working tree implementation diff, 10 files, 4,006 changed lines, with 3,993 additions and 13 deletions
**Verdict**: 4 defects, 0 smells, 1 nitpick

## Defects

### D5, inherited and fallback fixed-prefix shadows still corrupt modern comment XML
`crates/rpptx-oxml/src/comments.rs:1181`

The shadow-preservation scanner handles only `xmlns:p188` and `xmlns:a`, while
`model_attributes` also removes `xmlns:p188m`. An aliased list or model shell
can therefore own `xmlns:p188m="urn:producer"` for a preserved
`<p188m:producer/>` child. Serialization drops that declaration and either
leaves the raw child unbound or rebinds it to the writer's fallback model
prefix. Reopen can succeed while the preserved child's expanded name changes.

The DrawingML repair also stops at comments. `write_reply` decides whether to
restore `a` from only the reply's own raw attributes at
`crates/rpptx-oxml/src/comments.rs:920`. If a list-owned `xmlns:a` shadow is
retained for raw content, a typed reply text body inherits that producer URI
and its fixed `a:` children no longer belong to DrawingML. The new regression
covers a typed comment body under a comment-owned shadow, but covers neither a
typed reply under an inherited shadow nor a preserved `p188m` shadow.

### D17, aliased section shells drop producer-owned `p14` namespace bindings
`crates/rpptx-oxml/src/presentation.rs:1325`

`lexical_attributes` removes every `xmlns:p14` declaration without checking
its URI or whether preserved raw descendants depend on it. A valid aliased
`q:section`, `q:sldIdLst`, or `q:sldId` may own
`xmlns:p14="urn:producer"` and a raw `<p14:producer/>` child. A dirty section
write replaces that declaration with the model namespace while replaying the
raw child bytes unchanged, so the child's expanded name changes and reopen
succeeds. This is the section equivalent of the fixed-prefix corruption found
for comments.

### D18, removing or reordering section slide ids moves raw boundaries to the wrong side
`crates/rpptx-oxml/src/presentation.rs:1283`

The section writer emits slide-list raw children by the current numeric index
instead of reconciling original slide-id identities. For an original sequence
`256, <x:between/>, 900`, removing slide 256 writes slide 900 before
`<x:between/>`, even though repository boundary semantics anchor that raw child
to the next surviving original item. Reordering the public `slide_ids` vector
through a cloned `Section` has the same defect. The current regression removes
only the final id and contains no raw node between ids, so it cannot detect the
misplacement.

### D19, dirty section rewriting forgets inherited aliases used by section children
`crates/rpptx-oxml/src/presentation.rs:738`

The initial parser records only the namespace binding for the `sectionLst`
element's own prefix. The dirty rewriter later reparses the original list with
that reduced scope. If the presentation or extension owns a different alias
used by direct section children, such as a local `q:sectionLst` containing an
inherited `s:section`, the rewriter treats the old typed section as raw XML.
`set_sections` then preserves the old section and writes the replacement beside
it. Reopen exposes both sections and can commit a result that does not match the
requested replacement. Clearing sections through the same input retains the
old section unchanged.

## Smells

None.

## Nitpicks

- `crates/rpptx-oxml/src/comments.rs:86`, the author-list start branch computes and extends `raw_attributes` twice, then immediately shadows and discards the first result.

## Prior finding status

- D1 is remediated. Open rejects shared comment-part ownership, and commented-slide duplication is refused atomically.
- D2 is remediated. Section discovery requires the exact extension URI and direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place and retains its opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and write.
- D5 remains open as cited above. Parent-owned `p188` and `a` shadows are handled for the covered comment path, but the fallback `p188m` prefix and inherited reply text bodies remain unsafe.
- D6 is remediated. Direct text, CDATA, comments, processing instructions, and other raw events survive at comment model boundaries.
- D7 is remediated. Ordinary unsupported comment attributes retain their lexical source bytes.
- D8 is remediated. An actually self-closing presentation extension list expands in place.
- D9 is remediated for lexical bytes and direct nested events. D17 and D18 identify separate namespace-meaning and identity-boundary failures in the same typed section model.
- D10 is remediated. Public author, comment, and reply identifiers and timestamps are revalidated during serialization.
- D11 is remediated. The facade round-trip gate checks reply movement plus notes and handout header-footer values after reopen.
- D12 is remediated for generated children under a dirty aliased section list. D19 is a separate loss of other aliases inherited by the original child sections.
- D13 is remediated. Section insertion recognizes a self-closing extension-list root from its terminal lexical form, not a self-closing descendant.
- D14 is remediated for alias-only section lists. Namespace declarations bound to the section namespace no longer keep an empty typed shell.
- D15 is remediated. Author boundaries reconcile original author ids, so an appended author remains before the original trailing raw sidecar.
- D16 is remediated. The named facade round-trip gate asserts both saved collaboration content-type overrides.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Panics produced zero findings. Structure produced zero findings. Smells
produced zero findings. The only new module was explicitly approved, and the
diff adds no trait, generic, crate, feature, builder, forwarding wrapper, or
production dependency. Notes-master and handout-master schema ordering,
facade staging, comment-part ownership, relationship resolution, and new-part
content-type staging produced zero additional findings.
