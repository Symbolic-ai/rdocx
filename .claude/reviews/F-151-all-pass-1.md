# F-151, all, pass 1

**Reviewed**: uncommitted working-tree diff against `HEAD`, 12 files, 669 changed lines, with 648 additions and 21 deletions
**Verdict**: 4 defects, 1 smell, 0 nitpicks

## Defects

### D1, revision-wrapped floating drawings are omitted from both views
`crates/rdocx-layout/src/engine.rs:942`
`crates/rdocx-layout/src/engine.rs:1156`
`crates/rdocx-layout/src/engine.rs:1255`

The new run projection feeds inline layout, but anchored drawing collection and
the wrapping-drawing preflight still scan only `para.runs`. A floating drawing
inside `w:ins` or `w:moveTo` therefore disappears from the accepted and tracked
views. A drawing inside `w:del` or `w:moveFrom` also disappears from the tracked
view. Accepting the former revisions materializes the run and makes the drawing
appear, so accepted rendering is not equivalent to rendering the resolved
document. A projected wrapping anchor would also miss the reflow preflight.

### D2, heading outlines ignore the selected revision projection
`crates/rdocx-layout/src/engine.rs:267`
`crates/rdocx-oxml/src/text.rs:645`

The visible heading body now uses the accepted or tracked projected runs, but
the outline title still comes from `para.text()`, which does not include the
paragraph revision wrappers. A heading such as `Chapter <w:ins>Two</w:ins>`
renders as `Chapter Two` in the accepted body while its PDF outline omits
`Two`. After `accept_all`, the outline contains the inserted text, so the
accepted PDF view does not match the resolved document.

### D3, REF field text remains revision-view blind
`crates/rdocx-layout/src/engine.rs:824`
`crates/rdocx-layout/src/engine.rs:1141`

`bookmark_text` slices only the ordinary paragraph runs and has no revision
view input. When a bookmark spans inserted or moved-to content, the accepted
body includes that content but a REF field targeting the bookmark omits it.
After `accept_all`, the same REF field sees the materialized run and changes its
display. The tracked view also cannot include both sides of a revision in the
derived field value.

### D4, tracked decorations do not reach revised note references
`crates/rdocx-layout/src/engine.rs:695`
`crates/rdocx-layout/src/engine.rs:901`

The projection computes forced underline and strike values for the whole
revised run, but the footnote and endnote reference branch hard-codes no
underline and no strike. A `w:footnoteReference` or `w:endnoteReference` inside
an insertion is therefore not underlined in tracked view, and one inside a
deletion is not struck through. These markers are visible content from the
revision-wrapped run and fall under the tracked decoration contract.

## Smells

### S1, no test proves that a parsed revision produces a margin bar
`crates/rdocx-layout/src/paginator.rs:2407`
`crates/rdocx-layout/src/engine.rs:1743`
`crates/rdocx/tests/regression_test.rs:44`

The split-page paginator test sets `has_visible_revision` by hand, while the
property-only test calls only the predicate. The golden test merely requires
the tracked PNG to differ from the accepted PNG, which remains true from the
extra revision text and decorations even if every change bar is absent.
Removing the assignment that connects parsed revisions to `ParagraphBlock`
would therefore leave all three tests green. The test plan needs one assertion
that begins with revision XML and inspects the resulting page elements on each
split page.

## Nitpicks

None.

## Not found

Panics, OOXML preservation and schema ordering, and structural-rule violations
were checked and produced no findings.
