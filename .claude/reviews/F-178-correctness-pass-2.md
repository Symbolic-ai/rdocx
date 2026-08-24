# F-178, correctness, pass 2

**Reviewed**: the remediated F-178 working diff against `7bfa238`, 9
implementation and test files with 2,370 additions and 8 deletions, plus the
approved plan update
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, break runs bypass the projected run ceiling
`crates/rdocx/src/html.rs:1204`

Both an HTML `br` and each newline in preformatted text append a Word run that
contains `w:br`, but neither path calls `bump_runs`. One retained text node can
hold millions of newlines under the 64 MiB text limit and produce millions of
unaccounted OOXML runs. This bypasses the approved 100,000 projected-run bound.
The same omission occurs for preformatted newlines at
`crates/rdocx/src/html.rs:1218`.

### D2, comment contents can cause false DOM-limit rejection
`crates/rdocx/src/html.rs:333`

The revised preflight counts the opening comment marker, but it advances only
to the first `>` byte. Markup-looking text inside a single comment is then
counted as elements. A valid comment containing more than 100,000 repetitions
of `<x>` has one comment node in the parsed DOM but is rejected by the node
estimate. Comment scanning must advance through the closing `-->` marker while
still counting each separate comment node.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D1 is resolved by a limit-plus-one reader used by the path facade and
  a second concrete reader in the test gate.
- Pass 1 D2 is resolved by typed Word line-break runs, and the saved XML gate
  proves that a `w:br` reaches the package.
- Pass 1 D3 is resolved by a deterministic path-aware diagnostic for a table
  skipped inside inline list-item projection.
- Pass 1 D5 is resolved by ASCII case-insensitive relation-token matching and
  an uppercase regression input.
- No new contract, panic, OOXML child-order, dependency-direction, public API,
  test-vacuity, or structural finding was found beyond D1 and D2.
