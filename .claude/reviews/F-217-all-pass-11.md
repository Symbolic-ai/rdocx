# F-217, all, pass 11

**Reviewed**: uncommitted working tree implementation diff, 10 files, 4,997 changed lines, with 4,982 additions and 15 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D29, adding text moves a preserved extension list before the text body
`crates/rpptx-oxml/src/comments.rs:933`
`crates/rpptx-oxml/src/comments.rs:979`

A valid comment or reply may omit `txBody` and carry only the schema-final
`p:extLst` in its comment-properties group. The parser retains that unmodelled
extension at the boundary before a missing text body. Calling the public
`set_text` method then materializes a text body, but both writers emit that raw
boundary first and the new `txBody` second. The result reverses the required
`txBody`, `extLst` schema sequence. This is reachable through
`CommentList::from_xml`, followed by `comments[i].set_text`, for both a comment
and a reply. The current tests cover unchanged raw extension placement and new
text on constructor-built values, but not materializing text beside an existing
extension list.

## Smells

None.

## Nitpicks

None.

## Prior finding status

- D1 is remediated. Open rejects a comment part shared by slides, commented
  slide duplication is refused atomically, and slide removal retains an
  independently owned comment part.
- D2 is remediated. Section discovery requires the exact extension URI and a
  direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place while
  retaining its opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and
  write.
- D5 is remediated. The self-closing reply-list branch now carries local and
  inherited fixed-prefix shadows. Its exact regression passes.
- D6 is remediated. Direct text, CDATA, comments, processing instructions, and
  document types survive at author, comment, reply, and list boundaries.
- D7 is remediated. Unsupported comment-shell attributes retain their lexical
  source bytes.
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
  aliases, including a default namespace, used by direct section children.
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
- D25 is remediated. Slide removal serializes and reopens its staged graph
  before publishing section changes.
- D26 is remediated. Dirty section rewriting carries an inherited default P14
  binding through replacement and clearing.
- D27 is remediated. Typed unknown anchors retain their complete empty or
  nonempty source subtree byte for byte. Its exact regression passes.
- D28 is remediated. Notes and handout roots plus typed header-footer shells
  retain direct text, CDATA, comments, processing instructions, and document
  types at their schema boundaries. Its exact regression passes.
- The pass-5 nitpick remains remediated. Author-list parsing computes and
  stores the raw attribute sidecar once.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Correctness, OOXML, and test coverage produced D29. Smells produced zero
findings. Nitpicks produced zero findings. Panics produced zero findings.
Structure produced zero findings. The only new module was explicitly approved,
and the diff adds no trait, public generic, crate, feature, builder, forwarding
wrapper, or production dependency. The three exact pass-10 regressions pass.
The complete `rpptx-oxml` integration binary passes 114 tests. The two
environment-only `rpptx` oracle failures pass when rerun with a writable pinned
oracle cache. No additional defects were found in section identity
reconciliation, collaboration graph ownership, relationship resolution,
content-type creation, atomic facade staging, slide-removal staging,
notes-master or handout-master schema order, legacy-comment opacity, or
header-footer flag mutation.
