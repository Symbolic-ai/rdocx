# F-X028, correctness, pass 2

**Reviewed**: correction diff against prior branch tip `1865d23`, 3 files and
136 changed lines, comprising 102 additions and 34 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the generated archive glob is outside the path contract

`scripts/test_sprint_workflow.py:4871`

The standalone-path expression requires an alphanumeric first character and
does not accept the `crate` extension. It therefore omits the intentional
`*.crate` path claim in the verify command. The helper handles rooted globs and
the generated `target/package` root, but it does not yet cover every governed
path claim as the revised contract requires.

## Smells

None.

## Nitpicks

None.

## Not found

No further correctness, contract, panic, OOXML, test, or structure issues were
found. The non-crate and verify mutations are independent and fail for their
intended missing paths. Rooted globs, placeholders, line suffixes, and the two
known generated roots otherwise have explicit handling.
