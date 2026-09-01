# F-220, correctness, pass 5

**Reviewed**: claim base `3d87d827351508b38ffb22103f26351c3f18c0ca`
through exact source head `4b3db6a64c4bd4fd79c1677cb408249a0af6b76b`,
6 files and 4,237 changed lines, with 4,123 additions and 114 deletions. The
review includes the complete working diff, approved revised plan, cited HLD
sections, progress record, four prior untracked reviews, and all pass-4
remediation.
**Verdict**: 2 defects, 0 smells, 0 nitpicks. Not ready because both defects
remain and the external PowerPoint completion evidence is absent.

## Defects

### D1, missing authoritative data text falls back to a stale presentation copy
`crates/rpptx/src/diagram.rs:525`

The corrected precedence still uses the presentation point as a fallback when
the owning data point has no `CT_TextBody`. Missing data-node text is a valid
authoritative value. If the presentation point retains an old cached copy, this
path renders that copy even though the approved contract says `presOf` maps
authoritative data-node text and node text uses the F-219 data value. The new
regression distinguishes conflicting non-empty values, but its second owner
and presentation point both lack text at
`crates/rpptx/src/diagram.rs:1917`, so it does not distinguish `None` from a
foreign stale fallback.

### D2, SmartArt clips use the wrong coordinate space inside scaled parent groups
`crates/rpptx/src/lib.rs:7196`

Expansion records each frame's raw child-coordinate bounds at
`crates/rpptx/src/diagram.rs:161`. When the expanded diagram is nested in a
PresentationML group, flattening exposes a `group_scale`, but clip lookup drops
the rest of the `FlattenedItem` here and carries the raw rectangle forward.
The resolver scales each leaf's bounds by the parent-group scale, while
`smartart_local_clip` subtracts the unscaled frame origin from that scaled
shape origin and retains the unscaled frame width and height at
`crates/rpptx/src/lib.rs:7211`. For a parent group that doubles child
coordinates, a frame at x=10 with width=50 is clipped near x=10 with width=50
after its children resolve near x=20 at double size. The clip is therefore
offset and half-sized instead of matching the authoritative transformed frame.
Because the same prepared clip vector feeds static, timeline, media, and
animation lowering, all four paths share the error. The integration fixture
uses an ungrouped 400 by 200 frame and cannot expose parent scaling.

## Smells

None.

## Nitpicks

None.

## External completion evidence

The six-family PowerPoint oracle remains absent. Ordinary mode explicitly
skips at `crates/rpptx/tests/integration.rs:269` when
`f220-smartart-oracle.tsv` is missing. Required-corpus mode fails at
`crates/rpptx/tests/integration.rs:317` before comparing any family. The
progress record does not claim feature completion. Under the
differential-testing policy, this is an external feature-completion blocker,
not a defect, smell, or nitpick. Green source-built geometry, deterministic
Rust raster checks, and negative sensitivity do not establish the approved
PowerPoint 16.104, 1-point, and 0.99 SSIM acceptance gate.

## Not found

- **Pass-4 D1, non-empty authoritative text**: When both copies exist, the data
  owner's complete `CT_TextBody` now wins and retains its formatting. D1 is the
  adjacent missing-value case, not a recurrence of reversed non-empty
  precedence.
- **Pass-4 D2, complete node coverage**: Every ordinary or assistant data point
  must now own exactly one presentation point. Missing, duplicate, ambiguous,
  invalid-role, and cyclic topology continues to fail closed.
- **Pass-4 D3, literal and factor semantics**: Literal `val` is applied directly
  and `fact` scales the current extent. Multiple constraints execute in schema
  order, reject non-finite or out-of-bounds results, and no magnitude-dependent
  reinterpretation remains.
- **Pass-4 D4, ordinary clip plumbing**: Producing-scope maps separate slide,
  layout, and master identities. Top-level SmartArt clips are applied after the
  shared lowerer in static, timeline, media, and animation assembly, and an
  existing timeline clip is retained in a nested identity group. D2 is limited
  to missing parent-group coordinate conversion.
- **Pass-4 D5, exact shared-engine test**: The named unit now covers literal and
  factor order, unsupported rules, style references, ordered colour transforms,
  and retained text. The integration path additionally resolves formatted text
  and transformed colour through the shared engines.
- **Six-family geometry and graph bounds**: Exact family and algorithm pairs,
  parameter allowlists, typed ordinals, graph cardinality and depth, duplicate
  edges, cycles, finite geometry, and authoritative-bound containment remain
  checked. No new untrusted-input panic or unchecked arithmetic overflow was
  found.
- **OOXML namespace, schema, and preservation**: The doc-hidden projections
  continue to use schema-owned expanded-name paths, accept aliases, exclude
  fixed-prefix shadows and extension lookalikes, preserve ordered transforms,
  and retain source bytes exactly. Unsupported resource or layout input remains
  visible through a bounded placeholder or bounds fallback with a diagnostic.
- **Plan and API alignment**: Unsupported rules now fail closed consistently in
  implementation, plan, and tests. The two approved doc-hidden pre-1.0
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
