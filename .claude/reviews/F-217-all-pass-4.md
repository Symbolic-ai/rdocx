# F-217, all, pass 4

**Reviewed**: uncommitted working tree implementation diff, 10 files, 3,824 changed lines, with 3,811 additions and 13 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D5, fixed-prefix shadows owned above raw comment content still change its namespace
`crates/rpptx-oxml/src/comments.rs:290`

The remediation retains a conflicting `p188` or `a` declaration only on an
individual author, comment, or reply shell. The author-list, comment-list, and
reply-list roots still pass their attributes through `model_attributes`, which
discards every writer-owned namespace declaration at
`crates/rpptx-oxml/src/comments.rs:1026`. For example, a valid aliased
`q:cmLst` with `xmlns:p188="urn:producer"` and a direct preserved
`<p188:producer/>` child is rewritten with `p188` bound to the modern-comment
namespace. The raw child bytes survive, but their expanded name changes and
reopen succeeds. A comment shell that retains an inherited `xmlns:a` shadow
has the same problem when it also owns a typed text body, because the text-body
writer emits fixed `a:` children under that producer binding.

### D13, a nonempty presentation extension list is mistaken for a self-closing root
`crates/rpptx-oxml/src/presentation.rs:1068`

When no typed section list exists, insertion treats the extension-list root as
self-closing whenever any `/>` occurs anywhere in its bytes. A normal
`<p:extLst><p:ext uri="producer"/></p:extLst>` therefore selects the slash on
the descendant `p:ext`. The rewrite opens that child, inserts the section
extension there, writes an `extLst` closing tag, and then retains the original
closing tag. `CT_Presentation::to_xml` returns malformed nesting, while the
facade's atomic `set_sections` fails its reopen for a valid and common input.

### D14, clearing an alias-only section list retains an invalid empty typed list
`crates/rpptx-oxml/src/presentation.rs:1115`

`section_list_has_raw_content` ignores only the exact canonical
`xmlns:p14` declaration. An alias-only root such as
`<q:sectionLst xmlns:q=".../powerpoint/2010/main">` is consequently classified
as carrying producer payload. `set_sections(Vec::new())` preserves that shell
after removing its modelled sections, producing an empty `sectionLst` that the
same parser rejects because at least one section is required. The facade
therefore cannot clear sections from otherwise valid prefix-tolerant input.

### D15, appending an author moves a trailing raw sidecar before the new author
`crates/rpptx-oxml/src/comments.rs:121`

Author-list serialization emits each stored raw boundary immediately after
the author at the same numeric index, without reconciling original authors to
the current collection. If an opened author list ends with a preserved raw
child, `add_comment_author` pushes the new author but the old trailing child is
written after the old final author and before the new one. A schema-final raw
extension list then precedes a modelled author, and arbitrary producer content
no longer retains its trailing position. Comments and replies already use
identity-based boundary reconciliation, but authors do not.

### D16, the round-trip gate does not verify collaboration content types
`crates/rpptx/tests/integration.rs:5675`

The main round-trip test reopens through `Presentation` and checks only typed
facade values. The facade resolves authors and comments through relationships
without requiring their declared content types, so removing either new
content-type override from staging leaves this test and the other F-217 tests
green. The approved test plan explicitly requires the author and comment part
content types, as well as their package relationships, to be asserted after
save.

## Smells

None.

## Nitpicks

None.

## Prior finding status

- D1 is remediated. Shared comment-part ownership is rejected on open, and
  commented-slide duplication is atomic and refused.
- D2 is remediated. Section discovery requires the exact extension URI and a
  direct `p:ext` parent.
- D3 is remediated. A self-closing slide extension list expands in place while
  retaining its original opening bytes.
- D4 is remediated. Comment and reply status values are validated on parse and
  write.
- D5 remains open as cited above. The pass-3 parent-shell trigger is covered
  only for raw-only descendants, while list-owned shadows and typed DrawingML
  descendants remain unsafe.
- D6 is remediated. Direct text, CDATA, comments, processing instructions, and
  other captured events survive at comment model boundaries.
- D7 is remediated for ordinary unsupported comment attributes. Their source
  lexemes survive dirty serialization.
- D8 is remediated for an actually self-closing presentation extension list.
  D13 is a separate nonempty-list regression in that insertion path.
- D9 is remediated. Section, slide-id-list, and slide-id attributes and direct
  raw events retain their lexical bytes during dirty writes.
- D10 is remediated. Public author, comment, and reply identifiers and
  timestamps are revalidated during serialization.
- D11 is remediated. The facade round-trip test now checks reply movement plus
  notes and handout header-footer values after reopen.
- D12 is remediated for nonempty dirty aliased section lists. Generated
  children carry a local `p14` binding and reparse. D14 is a distinct clearing
  failure for the same class of prefix-tolerant input.

## Not found

Correctness, contract, panics, OOXML, tests, and structure were all checked.
Panics produced zero findings. Structure produced zero findings. The only new
module was explicitly approved, and the diff adds no trait, generic, crate,
feature, builder, forwarding wrapper, or production dependency. No additional
defects were found in notes-master or handout-master schema ordering, comment
graph ownership, section nested-sidecar preservation, or the fixed-prefix
binding added for nonempty aliased section lists.
