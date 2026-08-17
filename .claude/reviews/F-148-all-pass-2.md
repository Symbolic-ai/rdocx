# F-148, all aspects, pass 2

**Reviewed**: working tree against `HEAD`, 11 files and 1,836 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, a reference inserted inside a hyperlink becomes linked content
`crates/rdocx/src/comments.rs:519`

When the range end lies strictly inside an existing hyperlink span, insertion
increments only the span end. The new `w:commentReference` run is consequently
inside the hyperlink even though it was not part of the caller's original run
content. The span must be split around the inserted reference so both original
halves retain their relationship and the new anchor remains outside it.

### D2, a formatted comment reference run survives removal
`crates/rdocx/src/comments.rs:668`

The cleanup removes a reference-only run only when `properties` is absent.
Word commonly styles reference runs with run properties. Removing such a
comment strips `w:commentReference` but leaves an empty formatted run, so the
selected comment's reference anchor is not fully removed. Properties belonging
to a now-empty reference run should not keep that run alive.

## Smells

None.

## Nitpicks

None.

## Not found

The five pass 1 findings are remediated. No additional range validation,
thread linkage, comments-extended preservation, package graph, panic,
test-gate, or structural findings were found.
