# F-143, all, pass 4

**Reviewed**: fully remediated uncommitted `work/f-143-codex`
implementation, 12 files and 343 changed lines, including the two approved new
crate files
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Earlier findings

### Pass 1 D1 and pass 2 D1, resolved

Checked cumulative cardinality is charged before expansion at
`crates/oxml-cli-support/src/lib.rs:69`. Single oversized, disjoint, and
overlapping selections cannot amplify allocation or expansion work beyond
100,000 requested values.

### Pass 1 D2, resolved

The rdocx regression compares the complete legacy inspect payload inside the
schema-1 envelope at `crates/rdocx-cli/src/commands.rs:456` and executes the
real no-output conversion path at `crates/rdocx-cli/src/commands.rs:475`.

### Pass 3 D1, resolved

The exact documented maximum is accepted and checked for length and inclusive
endpoints at `crates/oxml-cli-support/src/lib.rs:150`. The neighboring tests
reject 100,001 values and cumulative overlapping work above the limit.

## Not found

- **Correctness**: no wrong range syntax, off-by-one, sorting, deduplication,
  output-path, JSON-envelope, or rdocx migration behavior was found.
- **Contract**: the implementation contains the three approved concrete
  helpers, migrates only rdocx inspect JSON and convert defaulting, and does not
  change flags or zero-based page indexing.
- **Panics**: checked cardinality arithmetic covers untrusted range values, and
  the counted-hyphen `expect` is protected by its branch invariant. No other
  new untrusted-input panic was found.
- **Tests**: the named range and schema gates, invalid and boundary matrices,
  resource bounds, output paths, complete inspect payload, and real default
  conversion path are mutation-sensitive to their intended behavior. Fresh
  runs passed all seven shared-crate tests, both rdocx-cli tests, and all 33
  sprint-workflow tests.
- **OOXML**: no package XML, namespace, child-order, whitespace, or unmodelled
  subtree behavior changes.
- **Structure and dependencies**: no trait, generic abstraction, builder,
  feature flag, forwarding wrapper, extra file, forbidden facade dependency,
  or duplicate shared helper was added.
- **Publication boundary**: package metadata, dependency pins, dry-run patches,
  release-family membership, and dependency-ordered allowlist agree. The diff
  prepares a future release path without creating a tag or publishing.
- **Harness and hygiene**: the change does not affect rendering or a baseline.
  Diff hygiene, prose validation, and generated-skill drift checks passed
  during review.
