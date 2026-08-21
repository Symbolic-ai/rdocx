# F-164, full, pass 1

**Reviewed**: working tree against `035d076`, 8 files, 1,161 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, row-only structural output is discarded when it has no scalar tags

`crates/rdocx/src/template.rs:172`

The final commit guard only observes body-level controls and scalar replacement
counts. A document whose only control is a table-row `if` or `for` can therefore
produce a changed row sequence with zero scalar leaves, then return `Ok(0)`
without committing the candidate. The method reports success while leaving the
original marker rows and content unchanged.

### D2, nested-table row markers are classified as markers on their outer row

`crates/rdocx/src/template.rs:363`

`row_marker` recursively collects text from the entire row, including nested
tables through `collect_cell`. A valid control row inside a nested table is
therefore presented to the outer table's parser as though the outer row itself
were the marker row. Any other outer-row text makes the template fail the
dedicated-row rule, while an otherwise empty outer row can be removed or cloned
at the wrong table boundary. Structural controls in body tables must remain
owned by the table that directly contains their marker rows.

### D3, scalar leaves in false conditional branches bypass preflight

`crates/rdocx/src/template.rs:563`

The evaluator skips the body of a false `if` without scanning its scalar tags.
A template containing a missing, non-scalar, or malformed scalar path inside
that branch therefore succeeds, even though the approved contract requires
preflight to validate all scalar leaves before replacing the typed document.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML preservation or schema-order,
test-gate, public-API, dependency, or structural-indirection findings were
found.
