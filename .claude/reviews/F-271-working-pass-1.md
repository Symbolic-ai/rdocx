# F-271, working, pass 1

**Reviewed**: uncommitted worker diff, 6 files, 293 added lines and 12 removed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, preservation test does not exercise the related-part comment edit

`crates/rdocx/tests/regression_test.rs:603`

The round-trip test inserts a separate paragraph beside an unknown subtree.
It never calls `add_story_comment`, so it passes even if the new paragraph
parser and serializer discard producer attributes or children inside a
paragraph being annotated. Put an unknown attribute and subtree inside the
selected paragraph, author a comment there, and assert both are retained.

## Smells

None.

## Nitpicks

None.

## Not found

The correctness, contract, panics, OOXML child order, and structure aspects
produced no other findings.
