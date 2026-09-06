# F-240, all aspects, pass 1

**Reviewed**: working diff from `539f37a1`, 7 files, 487 insertions and 35 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, complete rows still contain partial capability cells

`docs/hld/02-scope-and-non-goals.md:206`

The matrix defines `P` as only part of a family being modeled or public, but
DOCX-001, DOCX-002, DOCX-029, and DOCX-031 are classified `complete` while
retaining `P` cells. This makes the closed classification contradict the row
data. The regression accepts the contradiction because it validates only the
per-column vocabulary and special cases for the two preservation classes.

### D2, a boundary row can name a different owner than its evidence

`scripts/test_sprint_workflow.py:8065`

The owner regression proves that a partial or unsupported row contains one live
F-ID, but it does not prove that `boundary:F-XXX` names that same F-ID. A row can
therefore cite one story as its boundary and silently assign implementation to
another live story while the test remains green.

### D3, roadmap status alignment is not tested

`scripts/test_sprint_workflow.py:8092`

The roadmap regression proves HLD heading uniqueness, sprint-plan placement,
dependency endpoints, ordering, and cycles. It does not compare the canonical
BACKLOG sprint placement and status for F-240 through F-310 with the sprint
plan. The approved test contract explicitly includes status alignment, so a
story can move or disappear in BACKLOG while this gate remains green.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings.
The change adds no runtime parser or serializer, no public API, no crate edge,
and no schema-order or raw-XML preservation path.
