# F-275, default, pass 1

**Reviewed**: working diff against `976642e5ab44faa54a86db03f68dd85286f2d994`, 11 files, 1667 added and 71 removed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, nested body block controls cannot receive range endpoints
`crates/rdocx/src/document.rs:14784`

Every two-segment body endpoint takes the legacy typed paragraph path. A nested block control has a valid two-segment story item path, but `story_paragraph_mut` interprets the first segment as a direct body content index. It rejects the nested control or addresses another direct body item. The same branch also compares index paths lexically even when physical paragraph order differs. Route nested body controls through the source-backed resolver while preserving the existing direct block-control path and its diagnostics. Add a source-built nested body fixture with exact endpoints after reopen.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings in this pass. The related-owner nested-control and hidden-marker tests cover those paths.
