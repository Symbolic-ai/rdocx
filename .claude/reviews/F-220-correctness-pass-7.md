# F-220, correctness, pass 7

**Reviewed**: claim base `3d87d827351508b38ffb22103f26351c3f18c0ca`
through exact source head `7d3d5af637cb0c1b7755b2f2583a9945148e46e6`,
17 files and 5,925 changed lines, with 5,766 additions and 159 deletions. The
review includes the full committed delta, approved revised plan, cited HLD
sections, progress record, six prior untracked reviews, the user-approved
PowerPoint 16.104 cycle compatibility profile, and complete tracked and
untracked state.

**Verdict**: 2 defects, 0 smells, 0 nitpicks. Not ready.

## Defects

### D1, nested supported iteration has no total-work bound
`crates/rpptx/src/diagram.rs:503`

Program evaluation increments its semantic counters with unchecked `+=`, then
executes every child of a `forEach` once per selected node at
`crates/rpptx/src/diagram.rs:535`. `foreach_count` bounds only each individual
loop to the current node count or `MAX_NODES` at
`crates/rpptx/src/diagram.rs:621`. It does not bound the product of nested
loops, total visited instructions, generated-shape count, or condition count.
The projection and evaluator both permit a depth-valid chain of known
`forEach` instructions. With three nodes, nested loops grow as `3^depth` and
can consume unbounded CPU before overflowing a counter in checked builds.

The exact cycle resource is protected from this mutation by its raw-byte hash,
but the other five accepted identities are not runtime hash-gated. A modified
known-identity layout can therefore enter this path instead of failing closed.
This violates the approved requirement that iteration, instruction depth,
generated shapes, and condition evaluation are bounded before transient
allocation at `.claude/plans/F-220-design.md:158`. The tests exercise excessive
tree depth and one ordinary supported selector, but no nested multiplicative
iteration or total-work exhaustion case.

### D2, the finite instruction validator accepts duplicate parameters and unsupported values
`crates/rpptx/src/diagram.rs:370`

Instruction validation checks only whether attribute names occur in a broad
per-kind allowlist. It does not validate cardinality or the value contract for
parameters, shape types, constraint targets, selector combinations, variables,
or other known instruction kinds. Evaluation validates algorithm names and
some numeric fields at `crates/rpptx/src/diagram.rs:502`, but has no
`Parameter` arm at all. Later parameter lookup returns the first matching
parameter it encounters at `crates/rpptx/src/diagram.rs:1253`, so duplicate
parameters are accepted and resolved by source order instead of rejected.

This is reachable for the five non-cycle exact identities because family
inference uses only `uniqueId` and explicitly ignores both the flattened
algorithms and the parser's unsupported-layout result at
`crates/rpptx-oxml/src/diagram.rs:1956`. For example, an authentic relationship
program with two `ar` parameters renders using the first value. A known shape
instruction with an unsupported `type` is likewise ignored in favor of the
hard-coded family shape. Both inputs produce plausible geometry rather than
the required fallback. The plan requires the complete projected instruction
tree to match the supported finite subset and explicitly requires duplicate or
unsupported parameters and enum values to fail closed at
`.claude/plans/F-220-design.md:169` and
`.claude/plans/F-220-design.md:214`. The mutation regression changes a used
constraint and an instruction kind at `crates/rpptx/src/diagram.rs:2601`, but
does not challenge duplicate parameters or unsupported values on a known kind.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## External differential evidence

The progress record reports a complete six-family common-source PowerPoint
16.104 run with all geometry, symmetric text-masked SSIM, ordered text-line,
ownership, diagnostic, dimension, and sensitivity gates passing. The external
manifest and artifacts are not present in this review worktree. Ordinary mode
therefore skipped at `crates/rpptx/tests/integration.rs:360`, and required mode
failed at the same exact guard before comparing a family. This is recorded as
unavailable local external evidence, not counted as a source defect. The
source gate correctly fails closed when required evidence is absent.

## Not found

- **Cycle1 production identity and hash**: the cycle dispatch validates the
  projected program first, then requires exactly three semantic render nodes,
  the exact `cycle1` unique identity, and SHA-256 of the layout's preserved raw
  bytes at `crates/rpptx/src/diagram.rs:265` and
  `crates/rpptx/src/diagram.rs:342`. A same-projection byte change, changed
  identity, changed instruction, and different node count all reject. The
  pinned-resource regression passed when run explicitly with the installed
  PowerPoint resource.
- **Mutated layout state**: `CT_DiagramLayoutDefinition::to_xml` returns the
  private captured raw bytes at `crates/rpptx-oxml/src/diagram.rs:591`, while
  the render projection is exposed only by shared reference at
  `crates/rpptx-oxml/src/diagram.rs:595`. Mutating public compatibility fields
  cannot manufacture the pinned raw hash or mutate the private instruction
  root. Changing `unique_id` is caught explicitly. Changing another resource's
  family to Cycle still fails the raw hash. No profile bypass was found.
- **Cycle geometry, paths, and coordinates**: the three-node check precedes
  every fixed-array index. All six cycle rectangles are finite, positive, and
  contained in the normalized frame. Custom paths have finite bounded
  coordinates, are parsed through the existing checked
  `CT_CustomGeometry2D`, and use the shared geometry evaluator at
  `crates/rpptx/src/diagram.rs:1434` and
  `crates/rpptx/src/diagram.rs:1781`. The exact geometry test passed.
- **Passes 1 through 6**: cached drawing independence, presentation ownership,
  authoritative empty text, source and destination ordinals, duplicate graph
  edges, graph cycles, literal and factor constraint semantics, exact frame
  transforms, producing-scope separation, accumulated parent-group clipping,
  and static, timeline, media, and animation reuse remain corrected. No
  recurrence of those findings was found.
- **Frame bounds and panic safety**: frame and generated rectangles require
  finite positive dimensions, generated nodes must remain inside authoritative
  bounds, and fixed three-item family indexing follows exact node cardinality.
  Graph endpoints are validated before the remaining internal `expect` and map
  indexing paths. Apart from D1's unchecked work counters, no reachable
  untrusted-input panic, invalid slice, arithmetic overflow, or depth underflow
  was found.
- **OOXML namespace, schema, and preservation**: the nested projection remains
  expanded-name and schema-position aware, accepts namespace aliases, excludes
  fixed-prefix shadows and extension lookalikes, preserves child order and
  ordered colour transforms, and returns untouched source XML byte exact. The
  three focused projection tests passed.
- **Public API, structure, and dependencies**: the only new diagram module is
  the approved private `rpptx` module. The doc-hidden OXML instruction carriers
  match the revised pre-1.0 contract, while facade, layout, renderer, Python,
  WASM, and CLI surfaces remain unchanged. No trait, generic, feature, crate,
  dynamic dispatch, binary fixture, or integration binary was added. Although
  the worker claim-base diff repeats `sha2.workspace = true`, canonical S62
  integration head `577a5d6` already contains the same direct dependency from
  integrated F-218, so F-220 adds no dependency to the integrated result.
- **Focused validation**: the pinned cycle mutation regression passed
  explicitly. Private diagram tests passed 5 with 2 external-resource ignores.
  The SmartArt integration filter passed 18 with 2 generator ignores, including
  the negative geometry and pixel sensitivity case. The three OXML projection
  tests passed. `cargo test -p oxml-layout --no-default-features` passed 100
  tests and 3 doctests. Scoped `rpptx` and `rpptx-oxml` Clippy with warnings
  denied, formatting, and diff hygiene passed. These green cases do not
  exercise D1 or D2.
