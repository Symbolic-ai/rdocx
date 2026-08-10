# F-123, all, pass 1

**Reviewed**: working diff against claim base `d929b4e`, 1 file, 950 changed
lines with 933 insertions and 17 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no wrong formatting projection, parser state, duplicate handling,
  or series-slot behavior found.
- Contract: the diff implements the approved label collection, shared number
  format projection, series attachment, and external percentage-text gate
  without taking native label placement or broader format-language scope.
- Panics: no panic path on untrusted ChartML or public finite-value formatting
  input found.
- OOXML: no prefix, namespace, schema-order, XML-text validation, or raw subtree
  preservation defect found.
- Tests: the gate proves `0.25` with `0%` becomes viewer text `25%`. The complete
  required-corpus crate run passed 27 tests, including 34 label collections, 35
  axis number formats, fixed-prefix ordering, malformed input, raw preservation,
  and pinned LibreOffice 26.2.5.2 plus Poppler 26.01.0 evidence.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary indirection found.
