# F-164, full, pass 2

**Reviewed**: remediated working tree against `035d076`, 8 files, 1,333 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an outer marker row can hide and delete nested table content

`crates/rdocx/src/template.rs:386`

The remediated marker scan correctly excludes nested-table controls when it
decides whether the outer row is a marker, but it also excludes every other
nested-table source from the dedicated-row check. A row with a direct marker in
one cell and a populated nested table in another is therefore accepted as a
dedicated marker row. Evaluation removes the whole row and silently deletes the
nested table. Once a direct row marker is found, the own-row validation must
consider all descendant text so non-marker content cannot be discarded.

## Smells

None.

## Nitpicks

None.

## Not found

The pass-1 structural commit, nested-table ownership, and false-branch
preflight defects are otherwise remediated. No other correctness, contract,
panic, OOXML preservation or ordering, test-gate, public-API, dependency, or
structural findings were found.
