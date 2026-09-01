# F-220, correctness, pass 8

**Reviewed**: claim base `3d87d827351508b38ffb22103f26351c3f18c0ca`
through exact source head `aea56979038273b52455528d42403b33e2e1bcbc`,
17 files and 6,417 changed lines, with 6,258 additions and 159 deletions. The
review includes the full committed delta, approved revised plan, cited HLD
sections, progress record, seven prior untracked reviews, pass 7 remediation,
the cycle compatibility profile, and complete tracked and untracked state.

**Verdict**: 1 defect, 0 smells, 0 nitpicks. Not ready.

## Defects

### D1, non-cycle profiles still accept unsupported selector and ownership semantics
`crates/rpptx/src/diagram.rs:529`

The new per-identity validator has semantic arms for algorithms, shapes,
`forEach`, and conditions, but its catch-all accepts every other projected
record without checking its values, cardinality, order, or owner at
`crates/rpptx/src/diagram.rs:590`. This leaves `presOf`, constraints, rules,
variables, adjustments, layout-node ownership, and the global instruction
topology outside the finite identity profiles. The generic validator expressly
allows `axis`, `cnt`, `ptType`, `st`, and `step` on `presOf` at
`crates/rpptx/src/diagram.rs:449`, while evaluation only increments a mapping
counter at `crates/rpptx/src/diagram.rs:865`.

This is a plausible-geometry bypass, not merely an over-strict or unused check.
For example, changing an authentic list `<presOf />` to
`<presOf axis="unsupported" />` keeps the exact recognized identity because
family selection uses only `uniqueId` at
`crates/rpptx-oxml/src/diagram.rs:1956`. The altered record passes both
validators, satisfies the nonzero mapping count, and cannot affect the list
renderer because that renderer reads only two width constraints before emitting
its fixed family geometry at `crates/rpptx/src/diagram.rs:1457`. `forEach` has
an adjacent reference bypass: the presence of any `ref` value returns one
iteration before its axis, point type, count, start, step, or reference value is
validated at `crates/rpptx/src/diagram.rs:1008`.

The approved contract requires every selector, reference, constraint, rule,
ownership combination, and geometry form reached by each pinned tree to match
the supported finite subset at `.claude/plans/F-220-design.md:169`. Unsupported
tree shapes and parameters must fail closed at
`.claude/plans/F-220-design.md:214`. The new installed-resource regression
checks duplicate and unsupported parameters plus one shape enum at
`crates/rpptx/src/diagram.rs:2993`, but does not mutate `presOf`, `forEach`
reference and point selection, constraint or rule enums, or instruction-owner
topology. Those known-kind mutations can therefore still render plausible
output rather than the visible fallback.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 7 remediation verification

- **D1, shared total-work budget**: fixed. One checked budget is created before
  variable collection and is passed into recursive evaluation at
  `crates/rpptx/src/diagram.rs:802`. Collection charges every projected visit
  at `crates/rpptx/src/diagram.rs:821`, evaluation charges every executed visit
  at `crates/rpptx/src/diagram.rs:844`, and selected conditions charge the same
  budget before evaluation at `crates/rpptx/src/diagram.rs:917`. All semantic
  counters use checked addition at `crates/rpptx/src/diagram.rs:972`. The
  depth-valid twelve-level, three-way adversarial loop at
  `crates/rpptx/src/diagram.rs:2905` failed closed at the 65,536-unit bound in
  the focused run. No multiplicative execution path or unchecked semantic
  counter outside this shared evaluator budget was found.
- **D2, parameters and selected duplicates**: partially fixed. Direct parameter
  ownership, required type and value, and duplicate types within one algorithm
  fail closed at `crates/rpptx/src/diagram.rs:491`. Each supported algorithm
  now has ordered exact parameter tuples at `crates/rpptx/src/diagram.rs:674`,
  and numeric parameter lookup rejects multiple selected matches at
  `crates/rpptx/src/diagram.rs:1687`. The ordinary and installed-resource
  regressions for these cases passed. Shape values are family checked at
  `crates/rpptx/src/diagram.rs:610`. The remaining known-kind gap is D1 above.

## External differential evidence

The progress record reports a complete six-family common-source PowerPoint
16.104 run with all geometry, symmetric text-masked SSIM, ordered text-line,
ownership, diagnostic, dimension, and sensitivity gates passing. The external
manifest and artifacts are not present in this review worktree. Ordinary mode
therefore skipped at `crates/rpptx/tests/integration.rs:360`, and required mode
failed at the same guard before comparing a family. This is unavailable local
external evidence, reported separately and not counted as a source defect. The
missing-oracle gate correctly fails closed.

## Not found

- **Cycle compatibility profile**: the cycle dispatch validates the projected
  program, then requires exactly three semantic render nodes, the exact
  `cycle1` identity, and SHA-256 of preserved layout bytes at
  `crates/rpptx/src/diagram.rs:287` and `crates/rpptx/src/diagram.rs:368`.
  Identity, byte, instruction, and node-count mutations all rejected in the
  explicitly run installed-resource regression. No mutable-state, collision,
  coordinate, path, or node-cardinality bypass was found.
- **Graph, bounds, and panic safety**: graph node and connection limits precede
  presentation normalization at `crates/rpptx/src/diagram.rs:277`. The shared
  evaluator work and depth bounds reject recursive amplification, semantic
  counters use checked addition, and existing exact family cardinality checks
  precede fixed indexing. No fresh reachable panic, overflow, underflow,
  non-finite coordinate, invalid path, or transient allocation issue was found.
- **OOXML namespace, order, and preservation**: the instruction projection
  remains expanded-name and schema-owner aware, excludes extension lookalikes
  and namespace shadows, retains instruction and transform source order, and
  preserves untouched XML bytes. The three focused projection and depth tests
  passed. D1 concerns accepted semantics after correct projection, not raw-byte
  preservation.
- **Passes 1 through 6**: cached-drawing independence, authoritative text and
  empty text, presentation graph ownership and ordering, constraint factor
  semantics, frame transforms, producing-scope separation, nested clipping,
  static, timeline, media, and animation reuse remain corrected. No recurrence
  of those findings was found.
- **Public API and structure**: the SmartArt module remains private. The
  approved doc-hidden OXML projection carriers and facade behavior are
  unchanged by pass 7 remediation. No new trait, generic, feature, crate,
  module, dynamic dispatch, integration binary, or public API was added.
- **Integrated dependency context**: the worker delta records
  `sha2.workspace = true` at `crates/rpptx/Cargo.toml:52`, but canonical S62
  integration head `577a5d6a5465c7bcf99ae5674f646d201ec478fd` already contains
  that exact direct dependency from integrated F-218. F-220 adds no dependency
  to the integrated result. The canonical `quick-xml` line is an integration
  reconciliation item from F-218, not an F-220 dependency defect.
- **Focused validation**: private diagram tests passed 7 with 3 explicit
  external-resource ignores. The three ignored pinned-resource tests passed
  when run explicitly, including all six profiles, parameter and shape
  mutations, and the cycle compatibility gate. The SmartArt integration filter
  passed 18 with 2 generator ignores. The three OXML projection tests passed.
  `oxml-layout --no-default-features` passed 100 tests and 3 doctests. Scoped
  `rpptx` and `rpptx-oxml` library Clippy with warnings denied, formatting, and
  diff hygiene passed. All-target `rpptx` Clippy is currently blocked by three
  pre-existing F-221-only unused constants at
  `crates/rpptx/tests/integration.rs:4940`, outside the F-220 claim-base delta.
  These green cases do not exercise D1's selector and owner mutations.
