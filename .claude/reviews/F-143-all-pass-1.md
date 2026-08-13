# F-143, all, pass 1

**Reviewed**: uncommitted `work/f-143-codex` implementation, 12 files and 252
changed lines, including the two approved new crate files
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, range parsing can exhaust memory on a valid input

`crates/oxml-cli-support/src/lib.rs:53`

Every value in an inclusive range is inserted eagerly into a `BTreeSet` before
the result is collected into a second allocation. The public parser accepts the
full positive `usize` domain, so an input such as `1-18446744073709551615` on a
64-bit target is valid under the implemented grammar and then runs until the
process hangs or exhausts memory. Reject a range whose materialized cardinality
is not safely bounded, and add a regression that proves the rejection occurs
before expansion.

### D2, the rdocx compatibility regression does not protect the prior payload

`crates/rdocx-cli/src/commands.rs:455`

The test checks schema, file, and only the existence of `paragraphs`. Removing
`tables`, `content_elements`, `metadata`, or `styles_used` from the migrated
payload would leave this regression green, even though the design contract
requires every prior inspect field to survive. It also does not exercise the
default convert path named by the integration row. Assert the complete payload
shape and add coverage that detects a regression in the no-output convert path.

## Smells

None.

## Nitpicks

None.

## Not found

- **Correctness**: apart from D1, range syntax, sorting, deduplication, inclusive
  endpoints, output extension replacement, and JSON envelope behavior match the
  approved contract.
- **Contract**: the implementation stays within the approved shared helpers and
  migrates only rdocx inspect JSON and convert output selection. It does not
  change flags or page indexing.
- **Panics**: the counted-hyphen `expect` is protected by the immediately
  preceding branch invariant. No other untrusted-input panic was found.
- **OOXML**: no package XML, namespace, child-order, whitespace, or unmodelled
  subtree behavior changes.
- **Structure**: the new crate has the two explicitly approved paths and adds no
  trait, generic abstraction, builder, feature flag, forwarding wrapper, or
  facade dependency.
- **Dependencies and publication**: the new inward dependency direction is
  permitted, the release family and patched dry-run inventories agree, and the
  change prepares publication metadata without creating a tag or publishing.
- **Harness and hygiene**: no rendering behavior or baseline changed, generated
  skill adapters match their canonical command edits, and `git diff --check`
  passed during review.
