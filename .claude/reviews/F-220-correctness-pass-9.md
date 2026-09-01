# F-220, correctness, pass 9

**Reviewed**: claim base `3d87d827351508b38ffb22103f26351c3f18c0ca`
through exact source head `774e37f12ac66fbf7dbb2d5fa6867d4246de26bd`,
17 files and 7,657 changed lines, with 7,498 additions and 159 deletions. The
review includes the full committed delta, approved revised plan, cited HLD
sections, progress record, eight prior untracked reviews, the one-file pass 8
validator, the cycle compatibility profile, and complete tracked and untracked
state.

**Verdict**: 2 defects, 0 smells, 0 nitpicks. Not ready.

## Defects

### D1, the topology profile does not validate schema child order
`crates/rpptx/src/diagram.rs:816`

The topology validator checks child kinds and gives `choose` an `if` then
optional final `else` rule, but `layoutNode`, `forEach`, `if`, and `else` fall
through the catch-all without any sequence check at
`crates/rpptx/src/diagram.rs:851`. The per-layout-node profiles compare only
positional arrays of aggregate child-kind and semantic counts at
`crates/rpptx/src/diagram.rs:862`. They never compare the ordered child
sequence retained by the projection.

The parser does not reject this first. It projects every schema-owned child in
encounter order without a sequence-state check at
`crates/rpptx-oxml/src/diagram.rs:1348`. Reordering an authentic layout node's
existing algorithm, shape, presentation mapping, constraint list, or rule list
therefore preserves the same owner and all expected counts and passes the new
profile. Reordering the `if` children inside one branch does too. The input no
longer matches the pinned instruction shape or OOXML sequence, yet the
hard-coded family preparation can still emit plausible geometry.

The approved contract requires recognition of validated projected instruction
shapes at `.claude/plans/F-220-design.md:193`, and the projection contract
explicitly retains schema-owned nested instruction order at
`docs/hld/06-presentationml-model.md:539`. The pass 8 regression at
`crates/rpptx/src/diagram.rs:4213` mutates values, ownership, and one aggregate
cardinality, but has no same-cardinality reorder case.

### D2, aggregate profiles still accept extra and owner-mismatched executable records
`crates/rpptx/src/diagram.rs:538`

The semantic walk calculates the current layout owner, but its algorithm,
shape, and condition arms do not use that owner. Conditions validate only
`func` and `op` at `crates/rpptx/src/diagram.rs:600`, leaving the exact
`arg`, `axis`, `name`, `ptType`, and `val` combinations unchecked. Algorithms
and shapes likewise use family-wide allowlists rather than the exact owning
node and branch profile.

The topology walk compounds this gap. It expressly permits algorithms, shapes,
and presentation mappings under `if` and `else` at
`crates/rpptx/src/diagram.rs:693`, while direct profiles count only children of
the layout node at `crates/rpptx/src/diagram.rs:759`. The recursive semantic
counts include only constraints, rules, variables, and adjustments at
`crates/rpptx/src/diagram.rs:786`. An extra family-valid `<shape />` inside an
existing selected `if` therefore changes none of the expected profiles. It is
accepted, merely increments the evaluation shape counter at
`crates/rpptx/src/diagram.rs:2037`, and is ignored by the hard-coded family
geometry. For list, preparation still reads only two constraints and emits the
same six fixed shapes at `crates/rpptx/src/diagram.rs:2624`.

Even counted record kinds are not exact signatures. The helper used by rule
validation treats each field as optional at `crates/rpptx/src/diagram.rs:1322`,
so the list rule arm accepts a `primFontSz` rule after its authentic `for` or
`forName` field is removed at `crates/rpptx/src/diagram.rs:1603`. Replacing one
allowed constraint with a duplicate of another allowed constraint under the
same owner also preserves the aggregate count. Unselected conditions can carry
family-allowed but runtime-unsupported combinations without evaluation ever
reaching them. These mutations leave a known identity with a plausible render
instead of the required fallback.

The plan requires every record to be evaluated under its owning layout node
and iteration context and every unsupported field or ownership combination to
fail closed at `.claude/plans/F-220-design.md:169`. Aggregate category counts
and family-wide value sets do not establish a consumable exact profile for each
node and branch. The new regression does not add an extra valid branch child,
remove an otherwise optionalized authentic field, swap an allowed algorithm
between owners, or duplicate one allowed semantic signature while removing
another.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 8 remediation verification

- **Previously cited mutations**: fixed. The exact installed list `presOf`
  mutation, wrong-owner constraint, extra direct constraint, invalid rule,
  variable and adjustment values, invalid `forEach` reference, and duplicate
  layout-node owner all failed closed in the explicitly run regression at
  `crates/rpptx/src/diagram.rs:4213`. All five authentic non-cycle resources
  validate under the new profiles. D1 and D2 concern same-count order,
  signature, and branch-content cases that regression does not exercise.
- **Readable diagnostics**: the new validator names family, instruction kind,
  owner, selector, or cardinality for its covered failures at
  `crates/rpptx/src/diagram.rs:715` and
  `crates/rpptx/src/diagram.rs:1313`. No unclear covered-path diagnostic was
  found. The issue is that D1 and D2 never reach those diagnostics.

## External differential evidence

The progress record reports the unchanged six-family PowerPoint 16.104
common-source geometry, symmetric text-masked SSIM, ordered text-line,
ownership, diagnostic, dimension, and sensitivity gates passing. The manifest
and external artifacts are not present in this worktree. Ordinary mode skipped
at `crates/rpptx/tests/integration.rs:360`, and required mode failed at the same
guard before comparing any family. This is unavailable external evidence, kept
separate and not counted as a source defect. Required mode correctly fails
closed when the oracle is missing.

## Not found

- **Pass 7 total-work budget**: one checked budget still spans variable
  collection, every recursive executed instruction, generated-shape counters,
  and selected condition evaluation at `crates/rpptx/src/diagram.rs:1977`.
  The depth-valid twelve-level multiplicative regression passed. No evaluator
  budget bypass or unchecked semantic counter was found.
- **Cycle compatibility profile**: cycle remains excluded from the readable
  non-cycle profiles and is protected by exact identity, preserved-byte
  SHA-256, projected instruction validation, and exactly three semantic nodes
  at `crates/rpptx/src/diagram.rs:369`. Identity, byte, instruction, and node
  count mutations all rejected in the explicitly run installed-resource test.
- **Panics, graph bounds, and geometry bounds**: checked topology counters,
  projection and evaluator depth limits, graph node and connection limits,
  exact family cardinality, finite rectangles, and checked path construction
  remain intact. No fresh reachable panic, overflow, underflow, invalid fixed
  index, non-finite coordinate, or unbounded recursive evaluator path was
  found.
- **OOXML namespace and preservation**: projection remains expanded-name and
  schema-owner aware, excludes namespace shadows and extension lookalikes, and
  preserves untouched source bytes. The three focused projection tests passed.
  D1 is validation of the retained sequence, not a loss of source order.
- **Public API, dependencies, and structure**: the new validator is decomposed
  into private functions in the already approved private diagram module. It
  adds no public item, trait, generic, feature, crate, module, binary, or
  dynamic dispatch. Canonical S62 integration head
  `577a5d6a5465c7bcf99ae5674f646d201ec478fd` already contains the direct
  `sha2` dependency from F-218, so F-220 still adds no integrated dependency.
  Apart from the correctness omissions in D1 and D2, no separate structural
  smell was found.
- **Prior passes**: cached drawing independence, authoritative and empty text,
  graph ownership and ordering, literal and factor constraints, frame
  transforms, producing scopes, accumulated clipping, and static, timeline,
  media, and animation reuse remain corrected.
- **Focused validation**: private diagram tests passed 7 with 4 resource
  ignores. All 4 ignored resource tests passed when run explicitly. The
  SmartArt integration filter passed 18 with 2 generator ignores, including
  geometry and pixel sensitivity. The three OXML projection tests passed.
  `oxml-layout --no-default-features` passed 100 tests and 3 doctests.
  `rpptx` all-target, all-feature Clippy and scoped `rpptx-oxml` library Clippy
  passed with warnings denied. Formatting, diff hygiene, skill sync, and the
  unchanged 49 of 49 hash harness passed. These checks do not challenge D1 or
  D2.
