# F-X028, correctness, pass 3

**Reviewed**: corrected working implementation against prior branch tip
`1865d23`, 3 files and 136 changed lines, comprising 102 additions and 34
deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No correctness, contract, panic, OOXML, test, or structure issues were found.
The helper covers rooted paths and standalone tracked filenames in both
governed documents. Concrete paths resolve on disk, globs must match,
placeholders require an existing static prefix, numeric line suffixes are
removed before lookup, and only the three named generated-output claims bypass
checkout existence. Independent mutations reject stale crate, non-crate,
verify, version, feature, and package claims. The pass 2 archive-glob defect is
fixed and `*.crate` is now extracted and classified explicitly.
