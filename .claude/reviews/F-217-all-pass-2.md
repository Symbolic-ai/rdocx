# F-217, all, pass 2

**Reviewed**: uncommitted working tree implementation diff, 10 files, 3,178 changed lines, with 3,165 additions and 13 deletions
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, producer input can still alias one comment part across slides
`crates/rpptx/src/lib.rs:1398`

Opening validates relationship count per slide and global comment ids, but it
does not require each slide to own a distinct comment-part target. Two slides
can therefore reference the same empty modern comment list without triggering
duplicate-id validation. Removing either slide unconditionally deletes that
shared part and returns success, leaving the surviving slide relationship
dangling. The commented-slide duplication trigger from pass 1 is blocked, but
the per-slide ownership invariant is not yet enforced for opened packages.

### D3, expanding a self-closing slide extension list drops its preserved start tag
`crates/rpptx-oxml/src/slide_parts.rs:419`

The self-closing remediation replaces the complete existing `p:extLst` event
with a fresh attribute-free fixed-prefix element. A valid aliased extension
list, or one carrying producer or markup-compatibility attributes, loses those
namespace declarations and attributes when the first comment is added. The
duplicate extension list from pass 1 is gone, but the required in-place raw
preservation is still incomplete for the same trigger.

### D5, self-contained raw namespace shadows are rejected even when they are safe
`crates/rpptx-oxml/src/comments.rs:1055`

The new preflight rejects every element or attribute that uses `p188` or `a`
with a non-model namespace anywhere in the document. A preserved raw child
such as `<p188:producer xmlns:p188="urn:producer"/>` is self-contained and can
be replayed byte for byte without affecting a fixed-prefix modelled sibling,
but both author and comment parsing now reject it. Unused shadows on modelled
shells no longer corrupt output, while valid unsupported subtrees that own and
use their shadow cannot be opened.

### D8, sections cannot be added when the existing presentation extension list is self-closing
`crates/rpptx-oxml/src/presentation.rs:951`

For an existing `<p:extLst/>`, section insertion searches for a lexical closing
tag and returns `p:extLst has no closing tag`. `set_sections` therefore fails on
a valid presentation shape instead of expanding the existing schema-final
list, even though the equivalent slide extension case is explicitly handled.

### D9, typing the section list drops its direct unmodelled content
`crates/rpptx-oxml/src/presentation.rs:754`

The section-list parser captures a direct child, retains it only when it is a
modelled `p14:section`, and discards every other captured subtree. Empty raw
elements, comments, processing instructions, text, and root attributes are
also not stored. Because presentation parsing immediately canonicalizes the
typed list, an unrelated save loses producer content placed directly inside
`p14:sectionLst`, contrary to the repository preservation rule.

### D10, public comment fields bypass id and timestamp validation at write time
`crates/rpptx-oxml/src/comments.rs:737`

`Comment`, `CommentReply`, and `CommentAuthor` expose mutable public ids, author
ids, and timestamps. Their constructors validate initial values, but the
serializers validate only status before writing the current fields. A caller
can change `created` to `bad` or an id to a non-GUID and obtain schema-invalid
XML from `CommentList::to_xml` or `CommentAuthorList::to_xml`. The approved
contract requires caller-supplied ids and RFC 3339 timestamps to be validated,
and the low-level public roots must enforce that after field mutation as they
now do for status.

### D11, the round-trip gate does not assert the header-footer mutations or reply ordering
`crates/rpptx/tests/integration.rs:5655`

The facade test mutates notes and handout header-footer flags, then reopens the
package without reading either value. It also adds only one reply and never
calls `move_reply`. Dropping the dirty master serialization or implementing
reply movement with the wrong final-index semantics would leave the test green.
The named round-trip gate therefore does not prove all ordered collaboration
and navigation mutations in the approved test plan.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1 D2 is remediated. Section discovery now requires the exact extension
URI and direct parent shape.

Pass 1 D4 is remediated for status values. Parsing and writing both reject
values outside the declared enumeration.

Pass 1 D6 is remediated. Direct comments, processing instructions, text, and
other raw events are retained at author, comment, and reply-list boundaries.

Pass 1 D7 is remediated. Unsupported comment attributes retain their lexical
source bytes.

Panics produced no findings. The remaining `expect` sites are guarded by
states established in the same expression or branch.

Structure produced no findings. The only new module was explicitly approved,
and the diff adds no trait, generic, crate, feature, builder, forwarding
wrapper, or production dependency.
