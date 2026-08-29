# F-X062, working, pass 1

**Reviewed**: working-tree diff against claim Head
`22b8a207b8cc4c6f2212c827e4935f573fa53326`, 7 files, 549 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, restarted completion loses or misnumbers endnotes outside the rebuilt suffix

`crates/rdocx-layout/src/engine.rs:1405`

The completion branch calls `append_endnote_pages` on `recorded.pages` before
the retained prefix is attached at line 1411. The append helper discovers
endnotes only by scanning the pages it receives at
`crates/rdocx-layout/src/paginator.rs:1435`, and it starts their page numbers
from that partial vector length at
`crates/rdocx-layout/src/paginator.rs:1455`. If an endnote reference is in a
retained prefix and an edit near the document end has no reusable tail,
`recorded.stopped_at` is `None`, the scan cannot see the reference, and the
warm result omits the endnote pages. If a reference is in the rebuilt suffix,
the appended pages are numbered without the retained-prefix count. Either case
diverges from fresh pagination. The test at
`crates/rdocx-layout/src/engine.rs:8601` edits in the middle of a long exact
suffix, so it exercises cached-tail attachment rather than the promised
restarted-body-completion branch and does not catch this failure.

## Smells

None.

## Nitpicks

None.

## Not found

Contract scope, panic safety, OOXML schema order and preservation, and
structural-rule violations produced no additional findings. No new public API,
dependency, module, trait, generic, feature flag, or test binary was added.
