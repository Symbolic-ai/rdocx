# F-143, all, pass 3

**Reviewed**: twice-remediated uncommitted `work/f-143-codex`
implementation, 12 files and 335 changed lines, including the two approved new
crate files
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the documented range-limit boundary is not protected

`crates/oxml-cli-support/src/lib.rs:143`

The limit regressions cover 100,001 requested values and an overlapping total
of 100,002, but none proves that the documented maximum of exactly 100,000 is
accepted. Changing the comparison at `crates/oxml-cli-support/src/lib.rs:83`
from `>` to `>=` would reject `1-100000` while all six current shared-crate
tests remain green. Add the exact accepted boundary so the new public limit is
protected against this off-by-one regression.

## Smells

None.

## Nitpicks

None.

## Earlier findings

### Pass 1 D1 and pass 2 D1, resolved

Every component now charges its checked cardinality against one cumulative
100,000-value budget before any range expansion at
`crates/oxml-cli-support/src/lib.rs:69`. A single oversized range, a disjoint
union, and repeated overlapping ranges therefore cannot create unbounded
allocation or expansion work.

### Pass 1 D2, resolved

The rdocx regression still compares the complete legacy inspect payload inside
the schema-1 envelope at `crates/rdocx-cli/src/commands.rs:456` and executes the
real no-output conversion path at `crates/rdocx-cli/src/commands.rs:475`.

## Not found

- **Correctness**: range cardinality uses checked arithmetic, charges before
  expansion, sorts and deduplicates accepted values, and handles inclusive
  endpoints correctly. No production-logic defect was found.
- **Contract**: the three shared helpers, rdocx migration, package metadata,
  release-family membership, and publication dry-run patch remain within the
  approved scope. No flag, page indexing, tag, or publication action changes.
- **Panics**: the counted-hyphen `expect` is protected by its branch invariant.
  No untrusted-input overflow or other new panic was found.
- **Tests**: apart from D1, the named range and schema gates, rejection matrix,
  output-path matrix, resource bounds, complete inspect payload, and no-output
  conversion path prove their intended behavior. Fresh focused tests passed all
  six shared-crate tests and both rdocx-cli tests.
- **OOXML**: no package XML, namespace, child-order, whitespace, or unmodelled
  subtree behavior changes.
- **Structure**: no trait, generic abstraction, builder, feature flag,
  forwarding wrapper, extra file, or forbidden facade dependency was added.
- **Harness and hygiene**: the change does not affect rendering or a baseline.
  Diff hygiene, prose validation, and generated-skill drift checks passed
  during review.
