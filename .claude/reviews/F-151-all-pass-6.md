# F-151, all, pass 6

**Reviewed**: complete remediated working-tree diff against `HEAD`, 12 files, 1,278 changed lines, with 1,229 additions and 49 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, terminal hyperlink revisions sort after a following bookmark
`crates/rdocx-layout/src/engine.rs:49`
`crates/rdocx-layout/src/engine.rs:686`
`crates/rdocx-layout/src/engine.rs:1188`

Every hyperlink-owned revision receives `usize::MAX` as its raw position. If a
hyperlink contains an ordinary run, ends with an included revision, and is
followed by a bookmark at the same direct-run boundary, the revision is
actually before that bookmark. The projection instead treats it as after the
bookmark. REF text therefore includes revision text that lies before the
bookmark start, while the PAGEREF target marker is emitted before that text.
After accepting the revision, the materialized run sorts before the bookmark,
so the accepted render can disagree with the resolved document and PAGEREF can
name an earlier page.

### D2, an empty text child still produces a change bar
`crates/rdocx-layout/src/engine.rs:183`
`crates/rdocx-layout/src/engine.rs:778`

The visibility predicate treats every non-comment run child as visible, while
layout explicitly discards an empty text value. A wrapper such as
`<w:ins><w:r><w:t/></w:r></w:ins>` therefore contributes no positioned content
but still marks its paragraph and draws a tracked change bar. This is the same
visible-content contract that suppresses a genuinely empty wrapper.

### D3, the PDF save facade has no option-taking counterpart
`.claude/plans/F-151-design.md:40`
`crates/rdocx/src/document.rs:2858`

The approved plan requires option-taking PDF and raster methods beside the
existing methods. Every render family added a counterpart except `save_pdf`,
which still delegates unconditionally to the accepted default. A caller using
the native PDF save facade therefore cannot select the tracked view through
that method.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-5 D1 is resolved. Correctness and contract review produced D1 through D3
above. Panic safety, OOXML preservation and schema ordering, and
structural-rule compliance produced no additional findings. The focused
`rdocx-layout` unit suite and `rdocx` regression suite pass. The test aspect
produced no independent finding beyond the uncovered cases above.
