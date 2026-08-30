# F-217, all, pass 10

**Reviewed**: uncommitted working tree implementation diff, 10 files, 4,873 changed lines, with 4,858 additions and 15 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D5, self-closing reply lists still lose producer-owned fixed-prefix bindings
`crates/rpptx-oxml/src/comments.rs:660`

The self-closing `replyLst` branch retains unsupported attributes through
`model_attributes`, but it does not run either fixed-shadow preservation helper.
For example, an aliased
`<q:replyLst xmlns:p188="urn:producer" p188:flag='kept'/>` loses its namespace
declaration while retaining the qualified attribute. Dirty comment writing then
uses `p188` for the modern-comment model and replays `p188:flag` in the wrong
namespace. Reopen succeeds, so facade staging does not detect the semantic
corruption. The pass-7 shell-attribute remediation covered a nonempty reply
list, but the same prior D5 defect remains open for this distinct parser branch.

### D27, typed unknown anchors discard unsupported attributes and children
`crates/rpptx-oxml/src/comments.rs:597`
`crates/rpptx-oxml/src/comments.rs:641`

Both the start and empty anchor branches replace every correctly namespaced
`unknownAnchor` with the fixed bytes `<p188:unknownAnchor/>`. An anchor carrying
a producer attribute, namespace declaration, comment, or child subtree is
therefore collapsed during any ordered comment serialization. The approved
contract explicitly requires unsupported anchor properties, attributes, and
children to remain byte-exact. The round-trip gate uses only an attribute-free
self-closing anchor, so it does not exercise this loss.

### D28, notes and handout typing drops direct non-element raw XML
`crates/rpptx-oxml/src/notes_parts.rs:288`
`crates/rpptx-oxml/src/slide_parts.rs:1904`

The shared notes and handout root parser ignores direct text, CDATA, XML
comments, processing instructions, and document-type events. The header-footer
child capture helper does the same inside the newly typed `p:hf`. A handout
master containing `<!--keep-->` between `p:clrMap` and `p:hf`, or a notes-master
header-footer containing a producer processing instruction, loses that event
when the low-level root is serialized. Through the facade, requesting either
mutable header-footer marks the master dirty, so the next save also publishes
the loss. The tests preserve raw element subtrees but do not cover direct raw
events at either new boundary.

## Smells

None.

## Nitpicks

None.

## Prior finding status

- D1 is remediated. Open rejects shared comment-part ownership, commented slide
  duplication is refused atomically, and removing another slide retains the
  surviving comment owner.
- D2 is remediated. Section discovery requires the exact extension URI and a
  direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place and
  retains its opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and
  write.
- D5 remains open at the self-closing reply-list branch cited above. The other
  fixed-prefix candidates, inherited bindings, descendants, and covered shell
  attributes retain their meaning or fail closed.
- D6 is remediated at author, comment, reply, and list boundaries. D28 is a
  separate loss in the notes, handout, and header-footer parsers.
- D7 is remediated for ordinary comment-shell attributes. D27 is the distinct
  canonicalization of the typed anchor payload.
- D8 is remediated. An actually self-closing presentation extension list
  expands in place.
- D9 is remediated. Section, slide-id-list, and slide-id attributes and direct
  raw events retain their lexical bytes during supported dirty writes.
- D10 is remediated. Public author, comment, and reply identifiers and
  timestamps are revalidated during serialization.
- D11 is remediated. The facade round-trip gate checks reply movement plus
  notes-master and handout-master header-footer values after reopen.
- D12 is remediated. Generated children under dirty aliased and inherited
  default-namespaced section lists carry a model namespace binding and reparse.
- D13 is remediated. Section insertion recognizes a self-closing extension-list
  root from its terminal lexical form.
- D14 is remediated. Sidecar-free alias-only section lists clear without
  retaining an invalid empty typed shell.
- D15 is remediated. Author boundaries reconcile original author ids, so an
  appended author remains before the original trailing raw sidecar.
- D16 is remediated. The named facade round-trip gate asserts both saved
  collaboration content-type overrides.
- D17 is remediated. Inherited and local fixed-prefix bindings used by
  descendants or shell attributes survive, and exhausted candidates fail
  closed.
- D18 is remediated. Slide-list raw boundaries reconcile against original
  slide-id identity for removal and reorder.
- D19 is remediated. Dirty section replacement and clearing retain inherited
  nonempty aliases used by direct section children.
- D20 is remediated. Text-body root namespace declarations and unsupported
  root-attribute lexemes survive unchanged serialization and safe dirty writes.
- D21 is remediated. An unlinked conventional comment part carrying the
  matching modern-comment MIME blocks mutation atomically.
- D22 is remediated. A valid owned `comment1.xml` does not trigger the collision
  guard, and a second commented slide allocates `comment2.xml`.
- D23 is remediated. Clearing a section list with direct raw payload fails
  before mutation.
- D24 is remediated. Unchanged text bodies keep their raw bytes, while an unsafe
  dirty body depending on a producer-owned inherited `a` prefix fails closed.
- D25 is remediated. Slide removal serializes and reopens its staged graph before
  publishing section changes.
- D26 is remediated. Dirty section rewriting carries an inherited default P14
  binding, removes the original typed section during replacement, and clears a
  sidecar-free list. Both exact regressions pass.
- The pass-5 nitpick remains remediated. Author-list parsing computes and stores
  the raw attribute sidecar once.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Smells produced zero findings. Nitpicks produced zero findings. Panics produced
zero findings. Structure produced zero findings. The only new module was
explicitly approved, and the diff adds no trait, public generic, crate, feature,
builder, forwarding wrapper, or production dependency. The two exact D26
regressions pass. The complete `rpptx-oxml` integration binary passes 111 tests.
The `rpptx` integration binary passes 94 tests with 7 expected ignores. No
additional defects were found in section identity reconciliation, collaboration
graph ownership, relationship resolution, content-type creation, atomic facade
staging, slide-removal staging, notes-master or handout-master schema order,
legacy-comment opacity, or header-footer flag mutation.
