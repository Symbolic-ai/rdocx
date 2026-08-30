# F-217, all, pass 7

**Reviewed**: uncommitted working tree implementation diff, 10 files, 4,373 changed lines, with 4,360 additions and 13 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D5, fixed-prefix shadows used by unsupported attributes are still dropped
`crates/rpptx-oxml/src/comments.rs:1154`
`crates/rpptx-oxml/src/comments.rs:1082`

The dependent-shadow scanner examines only bytes after the modelled shell's
opening tag. It therefore does not see a preserved unsupported attribute that
uses the shell's producer-owned fixed prefix. For example, an aliased comment
with `xmlns:p188="urn:producer"` and `p188:flag="kept"` loses the namespace
declaration because no descendant element uses `p188`. The writer then selects
`p188` as its model prefix and replays `p188:flag` in the modern-comment
namespace. Reopen succeeds while the attribute's expanded name changes. The
same trigger applies to author, reply, and list shells, so the pass-6
descendant and exhausted-prefix coverage does not exhaust D5.

### D17, section shadows used by unsupported attributes are still not carried
`crates/rpptx-oxml/src/presentation.rs:1486`
`crates/rpptx-oxml/src/presentation.rs:1429`

Both the inherited and locally owned `p14` shadow scanners inspect only the
content after the typed shell's opening tag. A section list can therefore own
`xmlns:p14="urn:producer"` while an aliased typed section carries a preserved
`p14:flag="kept"` attribute and no `p14` descendant. Dirty writing locally
binds `p14` to the section model and replays the raw attribute under that new
binding. A locally declared producer binding on a section, slide-id list, or
slide-id shell fails the same way. Reparse succeeds while the attribute's
expanded name changes, so the pass-6 descendant and exhausted-prefix cases do
not exhaust D17.

### D20, typed comment text bodies drop their root namespace sidecar
`crates/rpptx-oxml/src/comments.rs:590`

The comment parser hands the complete `p188:txBody` subtree to `CT_TextBody`,
whose selected-root parser retains no attributes from that root. A text body
such as `q:txBody xmlns:x="urn:producer" x:flag='a&#x20;b'` with a preserved
`x:tail` child is written without either root attribute. The raw child remains
but its `x` prefix is now unbound. The facade's reopen check can still accept
the result because the delegated text parser dispatches nested content by
local name. This violates the approved byte-exact preservation contract for
unsupported comment attributes and children. The same path is used for reply
text bodies.

### D21, a correctly typed occupied conventional comment part does not fail
`crates/rpptx/src/lib.rs:1093`

The collision guard rejects `/ppt/comments/comment1.xml` only when its resolved
content type differs from the modern-comment type. If an unrelated unlinked
part occupies that path with `application/vnd.ms-powerpoint.comments+xml`, the
guard is bypassed, the numeric allocator chooses `comment2.xml`, and
`add_comment` commits successfully. The approved invalid-graph contract says
an occupied conventional part name fails atomically. Its current test uses an
untyped occupied part, so it does not cover this MIME-matching trigger.

## Smells

None.

## Nitpicks

None.

## Prior finding status

- D1 is remediated. Open rejects shared comment-part ownership, and commented-slide duplication is refused atomically.
- D2 is remediated. Section discovery requires the exact extension URI and direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place and retains its opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and write.
- D5 remains open as cited above. All three producer-owned model-prefix descendants now fail clearly, but fixed-prefix use by unsupported attributes is not detected.
- D6 is remediated. Direct text, CDATA, comments, processing instructions, and other raw events survive at comment model boundaries.
- D7 is remediated for ordinary unsupported comment-shell attributes. D20 identifies the separate typed text-body root loss.
- D8 is remediated. An actually self-closing presentation extension list expands in place.
- D9 is remediated. Section, slide-id-list, and slide-id attributes and direct raw events retain their lexical bytes when their namespace binding is not shadowed.
- D10 is remediated. Public author, comment, and reply identifiers and timestamps are revalidated during serialization.
- D11 is remediated. The facade round-trip gate checks reply movement plus notes and handout header-footer values after reopen.
- D12 is remediated. Generated children under a dirty aliased section list carry a model namespace binding and reparse.
- D13 is remediated. Section insertion recognizes a self-closing extension-list root from its terminal lexical form.
- D14 is remediated. Alias-only section lists clear without retaining an invalid empty typed shell.
- D15 is remediated. Author boundaries reconcile original author ids, so an appended author remains before the original trailing raw sidecar.
- D16 is remediated. The named facade round-trip gate asserts both saved collaboration content-type overrides.
- D17 remains open as cited above. Inherited descendant bindings and fully occupied fallbacks are handled, but inherited or local bindings used only by unsupported attributes are not.
- D18 is remediated. Slide-list raw boundaries reconcile against original slide-id identity for removal and reorder.
- D19 is remediated. Dirty section replacement and clearing retain inherited alternate aliases used by original direct section children.
- The pass-5 nitpick remains remediated. Author-list parsing computes and stores the raw attribute sidecar once.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Smells produced zero findings. Nitpicks produced zero findings. Panics produced
zero findings. Structure produced zero findings. The only new module was
explicitly approved, and the diff adds no trait, public generic, crate,
feature, builder, forwarding wrapper, or production dependency. The three
exact pass-6 focused regressions pass. No additional defects were found in
comment graph ownership, relationship resolution, section identity-boundary
reconciliation, inherited alternate section discovery, notes-master or
handout-master schema order, atomic facade staging, collaboration content-type
creation, or legacy-comment opacity.
