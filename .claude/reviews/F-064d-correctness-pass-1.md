# F-064d, correctness, pass 1

**Reviewed**: Working diff, 3 files and 435 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML, tests, and structure produced no
findings. The fixed nine slots accept only levels 1 through 9, the writer emits
modelled levels in schema order with fixed `a:` prefixes, unsupported subtrees
remain at their captured boundaries, and malformed numbered levels return
errors. The named round-trip gate exercises all nine slots and would fail if
the typed list-style implementation were reverted.
