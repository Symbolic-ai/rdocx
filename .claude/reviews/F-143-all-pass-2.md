# F-143, all, pass 2

**Reviewed**: remediated uncommitted `work/f-143-codex` implementation, 12
files and 327 changed lines, including the two approved new crate files
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, overlapping ranges still permit unbounded expansion work

`crates/oxml-cli-support/src/lib.rs:71`

The new cardinality guard bounds each component and the final unique set, but
every accepted component is still expanded in full before duplicate values are
discarded. Repeating `1-100000` within one input therefore performs another
100,000 tree insertions per short component while the unique count remains at
the accepted limit. A command-line-sized input containing thousands of those
overlapping components can still keep the parser busy for billions of tree
operations. Bound cumulative expansion work or merge intervals before
materialization, and add a regression whose overlapping ranges cannot amplify
work beyond the documented limit.

## Smells

None.

## Nitpicks

None.

## Earlier findings

### D1, unresolved

The range parser now rejects a single oversized component and a disjoint union
as soon as it exceeds 100,000 unique values. The overlapping-component case
described above still leaves the original resource-exhaustion class open.

### D2, resolved

The rdocx regression compares the complete inspect JSON object, including all
legacy top-level and metadata fields, at
`crates/rdocx-cli/src/commands.rs:456`. It also executes the real conversion
path without an output argument and verifies the independently expected `.md`
path and content at `crates/rdocx-cli/src/commands.rs:475`.

## Not found

- **Correctness**: apart from D1, the boundary check uses checked arithmetic,
  accepts exactly 100,000 unique values, and rejects larger single or disjoint
  selections before unbounded allocation.
- **Contract**: the three shared helpers, rdocx migration, package metadata,
  release-family membership, and publication dry-run patch remain within the
  approved scope. No tag or publication authority is exercised.
- **Panics**: the counted-hyphen `expect` remains protected by its branch
  invariant. The new checked cardinality calculation does not overflow.
- **Tests**: D2 is mutation-sensitive for a removed inspect field and a broken
  no-output conversion path. Fresh focused tests passed all five shared-crate
  tests and both rdocx-cli tests.
- **OOXML**: no package XML, namespace, child-order, whitespace, or unmodelled
  subtree behavior changes.
- **Structure**: no trait, generic abstraction, builder, feature flag,
  forwarding wrapper, extra file, or forbidden facade dependency was added.
- **Harness and hygiene**: the change does not affect rendering or a baseline.
  Diff hygiene, prose validation, and generated-skill drift checks passed
  during review.
