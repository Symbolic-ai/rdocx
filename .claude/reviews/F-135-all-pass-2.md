# F-135, all, pass 2

**Reviewed**: complete current working diff from claim base `e233385`, 7 files
and 793 added plus 33 removed lines, including the pass-1 review and approved
parity module
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1 is resolved. The exact ID set now contains 17 entries at
`crates/rdocx-py/tests/test_python_docx_parity.py:27`, and every entry points
to the raw upstream v1.2.0-tagged source root declared at line 6. Independent
downloads of all three tagged pages confirmed every recorded source statement.
Sixteen entries declare namespace-only transformation. The sole exception
retains the exact tagged held-row body at line 114 and adds exactly one public
`document.tables[0].rows[1]` re-fetch at line 23 before the second cell write.
The gate asserts both source and transformed constants at lines 528 through
535. Removing that re-fetch in a disposable copy reproduced the intended
`StaleElementError`. The line-spacing entry at line 178 now contains only the
two exact tagged statements and obtains the earlier page-local `Pt` name from
its setup.

Pass-1 D2 is resolved. `_line_spacing_record()` preserves `float` values as
`("relative", value)` and all other length values as `("length", int(value))`
at `crates/rdocx-py/tests/test_python_docx_parity.py:369`. Each writer authors
an absolute 18-point paragraph at line 473 and a distinct relative 1.75
paragraph at line 489. Both saved outputs are reopened by both readers, and
the exact pair is asserted at lines 615 through 617. Independently collapsing
the kind or changing 1.75 to 1.5 made the differential fail.

Pass-1 D3 is resolved. The common authoring helper sets
`LightShading-Accent1` at
`crates/rdocx-py/tests/test_python_docx_parity.py:491` for both writer calls.
The normalizer records that style through each reader at line 443, and the
saved-output loop asserts it after reopen at line 618. Removing the setter in a
disposable copy made that exact assertion fail.

No fresh oracle-pin, tagged-source provenance, manifest completeness,
transformation, strict-revision, writer-direction, reader-direction,
direct-writer-equality, paragraph, run, direct-formatting, table, cell,
relative or absolute spacing, style, unit, enum, normalization, package-byte,
XML, binary-fixture, runtime-dependency, approval, HLD-impact, WASM isolation,
formatting, prose, generated-skill, diff-hygiene, hash-expectation, or artifact
issue was found. Table width was specifically investigated and is correctly
absent from the shared parity record because python-docx 1.2.0 exposes no
public `Table.width` reader or writer. The record covers the complete common
documented surface rather than using XML to invent an oracle capability.

The isolated environment reported exact `python-docx==1.2.0`. Both focused
parity tests and all 33 binding tests passed. Independent tagged-source,
held-row re-fetch, spacing-kind, spacing-value, and table-style mutations each
failed the intended gate. Both outputs were read through rdocx and python-docx
before direct writer-record equality. No package or XML bytes were compared.
The binding crate and `rdocx-wasm` checks passed, as did formatting, prose,
skill sync, diff hygiene, source hashing, and artifact checks. The worker
records all 28 hash entries unchanged.
