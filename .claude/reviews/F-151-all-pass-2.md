# F-151, all, pass 2

**Reviewed**: complete remediated working-tree diff against `HEAD`, 12 files, 865 changed lines, with 819 additions and 46 deletions
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, collapsed comment runs leave bookmark raw ordering stale
`crates/rdocx-oxml/src/text.rs:889`
`crates/rdocx-layout/src/engine.rs:1154`

The remediation adds `BookmarkMarker::raw_before` so projected revisions can be
compared with bookmark endpoints at one run boundary. When
`remove_comment_anchors` deletes a comment-reference-only run and collapses two
boundaries, it remaps each bookmark's `run_index` but not its new `raw_before`.
The revision raw slots immediately below are remapped with the required raw
prefix. A bookmark end after such a revision can therefore collapse onto the
same tuple as the revision. The exclusive `position < end` comparison then
drops that revision from REF text after comment removal even though the
bookmark still encloses it in document order.

### D2, accepted revised note markers can differ from accepted-and-resolved output
`crates/rdocx-layout/src/engine.rs:903`
`crates/rdocx-layout/src/engine.rs:917`

`revision_marker` is based on wrapper provenance rather than the selected
tracked view. An inserted footnote or endnote reference rendered with
`RevisionView::Accepted` therefore receives its direct underline, double
strike, and highlight properties. After `accept_all`, the same run is ordinary
and this branch suppresses those properties. The accepted pixels can differ
from the resolved document for a formatted inserted note reference, contrary
to the accepted-view gate. The forced tracked underline and strike themselves
are now propagated correctly.

## Smells

### S1, the split-bar test does not assert unchanged text placement
`crates/rdocx-layout/src/engine.rs:1855`
`crates/rdocx-layout/src/engine.rs:1862`

The remediated test now starts from parsed revision XML and inspects the actual
paginated result, which closes the pass-1 wiring gap. It only filters and checks
the margin bars, however. The design plan also requires the bar to leave text
placement unchanged. There is no control layout or positioned-text comparison,
so a later implementation that shifts paragraph text while still drawing one
correct bar per page would pass this test.

## Nitpicks

None.

## Not found

Pass-1 D1, D2, D4, and S1 are fully resolved. The direct view-blind slicing in
pass-1 D3 is resolved, with the mutation-specific ordering defect recorded
above. Panics, OOXML preservation and schema ordering, and structural-rule
violations were checked and produced no additional findings.
