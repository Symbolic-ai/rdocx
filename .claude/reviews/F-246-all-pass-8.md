# F-246, all, pass 8

**Reviewed**: formal changed-crate test failure against the verified working-tree diff, 20 files, 3,153 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, fragment collision regression expects an obsolete canonical link prefix
`crates/rdocx/tests/regression_test.rs:13278`

The fixture deliberately imports an aliased `s:link` element. F-246 promotes
that reference to the typed linked-style field while preserving the producer
element form and remapping its value. The assertion still requires a generated
`w:link`, so it rejects the correct preserved `s:link` even though the output
contains exactly one remapped link for each imported style. Assert the aliased
preserved form and typed linked-style targets.

## Smells

None.

## Nitpicks

None.

## Not found

The full library and integration suites, including the pinned LibreOffice and
Poppler style differential, passed. No production correctness, contract,
panic, OOXML ordering, preservation, or structure finding accompanied this
stale regression expectation.
