# F-220, correctness, pass 6

**Reviewed**: claim base `3d87d827351508b38ffb22103f26351c3f18c0ca`
through exact source head `c6c1cb648b22c775df00bf920c92b97144d2fe44`,
6 files and 4,322 changed lines, with 4,208 additions and 114 deletions. The
review includes the complete working diff, approved revised plan, cited HLD
sections, progress record, five prior untracked reviews, and all pass-5
remediation.
**Verdict**: 0 defects, 0 smells, 0 nitpicks. The reviewed source is ready for
the next workflow step. The external PowerPoint completion gate remains
separate and incomplete.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## External completion evidence

The six-family PowerPoint oracle remains absent. Ordinary mode explicitly
skips at `crates/rpptx/tests/integration.rs:345` when
`f220-smartart-oracle.tsv` is missing. Required-corpus mode fails at the same
guard before comparing any family. The progress record does not claim feature
completion. Under the differential-testing policy, this remains an external
feature-completion blocker, not a defect, smell, or nitpick. Green source-built
geometry, deterministic Rust raster checks, and negative sensitivity do not
establish the approved PowerPoint 16.104, 1-point, and 0.99 SSIM acceptance
gate.

## Not found

- **Pass-5 D1, empty authoritative text**: Render nodes now take only the data
  owner's optional `CT_TextBody` at `crates/rpptx/src/diagram.rs:525`. A stale
  presentation-point body cannot fill a missing owner value, while a present
  owner body remains complete and formatted. The regression now supplies stale
  presentation text for both a populated and a missing owner and asserts the
  latter remains empty at `crates/rpptx/src/diagram.rs:1934`.
- **Pass-5 D2, accumulated group scale**: Clip lookup retains the flattened
  leaf's accumulated `group_scale` and applies it to frame origin and extent at
  `crates/rpptx/src/lib.rs:7217`. This matches the resolver's scaled leaf bounds
  before their shared rigid or affine parent transform. Nested translations,
  rotations, flips, child offsets, uniform and non-uniform scales, and the
  existing shear fallback therefore keep the frame and child in the same local
  coordinate space. Invalid or non-positive clip geometry fails closed instead
  of reaching the lowerer.
- **Static, timeline, media, and animation clipping**: The producing-scope
  integration now nests the slide SmartArt frame in a scaled group and checks
  the exact scaled clip in static, timeline, and media output at
  `crates/rpptx/tests/integration.rs:165`. Animation uses the same prepared
  timeline request and clip application before encoding, and its edited output
  and diagnostics remain covered. Existing timeline clips are still composed
  by nesting rather than overwritten.
- **Producing-scope identity**: Slide, layout, and master expansion retain
  separate clip maps, so equal non-visual IDs cannot alias across scopes.
  Allocated transient child IDs remain unique within each complete shape tree.
- **Topology and authoritative content**: Every ordinary or assistant data
  point must own exactly one presentation point. Missing, duplicate, ambiguous,
  invalid-role, and cyclic ownership or topology fail closed. Data-node text,
  style labels, colour choices, and transforms continue through the shared
  DrawingML engines.
- **Constraint and six-family semantics**: Literal `val`, multiplicative
  `fact`, schema order, exact family and algorithm pairs, parameter allowlists,
  typed ordinals, graph cardinality and depth, duplicate edges, finite geometry,
  and authoritative-bound containment remain checked. Unsupported rules and
  ownership remain fail closed. No new untrusted-input panic or unchecked
  arithmetic overflow was found.
- **OOXML namespace, schema, and preservation**: The doc-hidden projections
  continue to use schema-owned expanded-name paths, accept aliases, exclude
  fixed-prefix shadows and extension lookalikes, preserve ordered transforms,
  and retain source bytes exactly. Unsupported resource or layout input remains
  visible through a bounded placeholder or bounds fallback with a diagnostic.
- **Plan and API alignment**: The plan now states the accumulated parent-group
  scale used by the implementation. The two approved doc-hidden pre-1.0
  `rpptx-oxml` projections remain additive. The `rpptx` facade, renderer,
  Python, WASM, and CLI public APIs remain unchanged.
- **Structure and dependency direction**: The single approved private module
  remains in `rpptx`. No new trait, generic, feature, crate, dependency, dynamic
  dispatch, integration binary, or binary fixture was added, and parsing and
  rendering boundaries remain intact.
- **Focused validation**: `cargo test -p rpptx --lib diagram::tests` passed 11
  tests. The three focused `rpptx-oxml` projection tests passed.
  `cargo test -p rpptx --test integration smartart_ -- --nocapture` passed 18
  tests with one ignored generator and the documented missing-oracle skip.
  `cargo test -p oxml-layout --no-default-features` passed 100 tests and 3
  doctests. `cargo check -p rpptx --all-targets`, the 49-of-49 hash harness,
  and `git diff --check` passed. Required corpus mode failed at the absent
  manifest as designed.
