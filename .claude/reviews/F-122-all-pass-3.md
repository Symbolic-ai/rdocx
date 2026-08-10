# F-122, all, pass 3

**Reviewed**: working diff from claim base `ff1e9c4`, 2 implementation files
and 1,560 changed lines, comprising 1,508 additions and 52 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 2 remediation

- Pass 2 D1 is fixed. Schema-routed raw nodes now advance the live boundary at
  `crates/rpptx-chart/src/lib.rs:4937`, so following comments, processing
  instructions, and whitespace remain after the routed node.
- The direct pie regression retains `<!--after-ext-->` after `c:extLst` at
  `crates/rpptx-chart/src/lib.rs:7436` while inserting labels before the
  extension.
- The direct area regression retains `<!--after-drop-lines-->` after
  `c:dropLines` at `crates/rpptx-chart/src/lib.rs:7431` while inserting labels
  before the drop lines.
- Pass 2 D2 is fixed. Plot validation rejects both public typed bubble-size
  data and private opaque bubble-size state at
  `crates/rpptx-chart/src/lib.rs:2348`.
- The malformed-input regression constructs an opaque `c:bubbleSize` wrapper
  at `crates/rpptx-chart/src/lib.rs:7285` and proves that a supported pie plot
  rejects it.

Both direct remediation tests passed in this review. The implementing session
also reports the full required 40-test corpus and viewer suite, formatting, and
strict workspace Clippy as green.

## Not found

- Correctness: no wrong enum mapping, default, range check, scatter-cache
  mapping, plot validation, axis resolution, or reciprocal-axis defect was
  found.
- Contract: pie, doughnut, area, scatter, and radar are typed within the
  approved boundary. Unsupported and combination plots remain opaque, and no
  F-125 native geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML: no namespace-alias, fixed-prefix, modelled-child sequence, repeated
  collection, raw-boundary, comment-order, unsupported-family preservation, or
  unknown-attribute defect was found.
- Tests: the gate, ordering, standalone scatter, malformed-value, typed and
  opaque bubble-size, mutation, preservation, corpus, and pinned viewer paths
  are exercised.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.
