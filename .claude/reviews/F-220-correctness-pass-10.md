# F-220, correctness, pass 10

**Reviewed**: claim base `3d87d827351508b38ffb22103f26351c3f18c0ca`
through exact source head `cdad43c5e0f5772f62166ff447d9c2996a1524d1`,
17 files and 8,427 changed lines, with 8,268 additions and 159 deletions. The
review includes the full committed delta, approved revised plan, cited HLD
sections, progress record, all nine prior untracked reviews, the final one-file
validator revision, cycle compatibility profile, and complete tracked and
untracked state.

**Verdict**: 2 defects, 0 smells, 0 nitpicks. Not ready at the final allowed
review boundary.

## Defects

### D1, ordered signatures cannot distinguish reordered same-kind children
`crates/rpptx/src/diagram.rs:1089`

The new signature records only each child's instruction kind. This fixes a
`shape,presOf` to `presOf,shape` reorder, but gives every constraint the same
`constr` token, every condition the same `if` token, and every iteration the
same `forEach` token. A constraint list then validates only that every child is
a constraint and that the total count matches at
`crates/rpptx/src/diagram.rs:1141`. It does not bind ordered constraint value
signatures to their occurrence indices.

For example, two authentic list constraints can be exchanged without changing
`constr,constr,...`, the owner, total count, individual field validity, or any
other profile. Reordering two of the relationship root's repeated `forEach`
children has the same problem because the root profile contains seven
indistinguishable `forEach` tokens at `crates/rpptx/src/diagram.rs:1523`, while
the `forEach` child-profile lookup receives no occurrence path at
`crates/rpptx/src/diagram.rs:1299`. The input no longer matches the exact pinned
instruction order but can still reach the hard-coded family renderer.

The same gap applies inside a multi-branch `choose`. Its expected signature is
only `if,if,if,if,if,else` at `crates/rpptx/src/diagram.rs:1384`. Swapping two
otherwise accepted condition tuples without moving their child constraint
lists preserves that signature and every branch-local count. The selected
condition then owns the wrong constraint set, or the hard-coded renderer simply
ignores the changed instruction ordering and emits plausible geometry.

The contract recognizes exact validated projected instruction shapes at
`.claude/plans/F-220-design.md:193`, and the HLD makes nested instruction order
part of the retained projection at `docs/hld/06-presentationml-model.md:539`.
The reorder regression at `crates/rpptx/src/diagram.rs:5004` exchanges two
different kinds, so it cannot detect this same-kind failure.

### D2, occurrence paths still do not bind every authentic field and value tuple
`crates/rpptx/src/diagram.rs:544`

The recursive walks construct useful occurrence-qualified paths, but several
instruction validators do not consume them. Constraint validation has no path
parameter at all at `crates/rpptx/src/diagram.rs:2014`. It checks one
family-wide set of allowed field names and values plus a broad owner rule. The
exact occurrence and the complete expected constraint signature are never
matched. Replacing an unused list constraint with a duplicate of another
family-valid `linear` constraint therefore preserves the count and passes all
checks, while list rendering still reads only the two width factors at
`crates/rpptx/src/diagram.rs:3352` and produces plausible output.

Algorithm validation receives a path, but `allowed_algorithm_owner` uses it
only for Pyramid at `crates/rpptx/src/diagram.rs:737`. For List, all four `lin`
parameter tuples are accepted family-wide at
`crates/rpptx/src/diagram.rs:2584`, and both `linear` branches accept any such
tuple. Changing the selected normal-direction branch from the authentic left
alignment tuple to the equally allowed right alignment tuple therefore passes
the exact parameter, owner, container, and path checks. Evaluation only counts
the algorithm, and the list renderer does not consume that parameter tuple.

Other instruction classes retain the same occurrence ambiguity. Condition
validation accepts ranges of family values rather than the exact tuple for each
path at `crates/rpptx/src/diagram.rs:798`. Layout-node validation allows any
family style label on any named owner at `crates/rpptx/src/diagram.rs:1667`,
while the topology profile contains no node-attribute signature. Adding an
allowed `node1` style to the root list node passes and is ignored by prepared
geometry. These are all valid-looking, known-class mutations that should fail
closed under the plan's every-record owner and iteration-context requirement at
`.claude/plans/F-220-design.md:169`.

The seven new adversarial cases at `crates/rpptx/src/diagram.rs:5004` cover a
different-kind reorder, extra branch shape and presentation mapping,
wrong-owner algorithm and mapping, and missing constraint and rule fields.
They do not cover same-kind reorder, a family-valid parameter tuple at the
wrong occurrence, duplicate substitution with unchanged count, a condition
tuple moved between occurrences, or a family-valid layout-node attribute at
the wrong owner.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 9 remediation verification

- **D1 covered cases**: the validator now compares ordered direct child kinds
  for layout nodes, iterations, choices, branches, shapes, variable lists,
  adjustment lists, and leaf records at `crates/rpptx/src/diagram.rs:1128`.
  The different-kind reorder fails as intended. D1 is the remaining
  indistinguishable same-kind case.
- **D2 covered cases**: exact owner checks now cover shape values and algorithm
  kinds, condition fields are finite, rules require complete authentic tuples,
  constraint field sets are required, variable order is exact, and branch
  child cardinalities are checked. All seven new adversarial mutations failed
  closed in the explicitly run installed-resource regression. D2 is the
  remaining occurrence-specific value and duplicate-substitution gap.
- **Diagnostics**: all covered failures name the family, owner, path, field
  set, child signature, or rejected semantics at
  `crates/rpptx/src/diagram.rs:593` and
  `crates/rpptx/src/diagram.rs:1186`. No unclear diagnostic was found for a
  path the validator actually rejects. D1 and D2 bypass diagnostics entirely.

## External differential evidence

The progress record reports the unchanged six-family PowerPoint 16.104
common-source geometry, symmetric text-masked SSIM, ordered text-line,
ownership, diagnostic, dimension, and sensitivity gates passing. The external
manifest and artifacts are absent from this worktree. Ordinary mode skipped at
`crates/rpptx/tests/integration.rs:360`, and required mode failed at the same
guard before comparing a family. This is unavailable local external evidence,
kept separate and not counted as a source defect. Missing-oracle handling
correctly fails closed.

## Not found

- **Total-work budget**: the one checked budget still covers variable
  collection and every recursively executed instruction and condition at
  `crates/rpptx/src/diagram.rs:2705`. Semantic counters remain checked. The
  twelve-level multiplicative regression at
  `crates/rpptx/src/diagram.rs:4808` passed. No fresh budget or depth bypass was
  found.
- **Cycle compatibility profile**: cycle remains outside the readable
  non-cycle profile and still requires exact identity, preserved-byte SHA-256,
  validated projection, and exactly three semantic nodes at
  `crates/rpptx/src/diagram.rs:369`. Identity, byte, instruction, and node-count
  mutations all rejected in the installed-resource test.
- **Panics and bounds**: graph node and connection bounds, projection and
  evaluator depth limits, checked counters, exact family node cardinality,
  finite rectangles, path parsing, and frame containment remain intact. No
  fresh reachable panic, overflow, underflow, invalid fixed index, non-finite
  coordinate, or unbounded recursive evaluator path was found.
- **OOXML namespace and preservation**: projection remains expanded-name and
  schema-owner aware, excludes namespace shadows and extension lookalikes,
  retains source order, and serializes untouched layout XML byte exact. The
  three focused projection tests passed. D1 and D2 are acceptance-profile
  defects after correct projection, not preservation loss.
- **Public API, dependencies, and structure**: the validator remains private
  in the approved diagram module. It adds no public item, trait, generic,
  feature, crate, module, binary, dynamic dispatch, or oracle dependency.
  Canonical S62 head `577a5d6a5465c7bcf99ae5674f646d201ec478fd`
  already contains the direct `sha2` dependency from F-218, so F-220 adds no
  integrated dependency. Apart from D1 and D2's profile omissions, no separate
  structural smell was found.
- **Prior fixes**: cached drawing independence, authoritative and empty text,
  graph ownership and ordering, literal and factor constraints, frame
  transforms, producing-scope isolation, accumulated clipping, static,
  timeline, media, and animation reuse remain corrected.
- **Focused validation**: private diagram tests passed 7 with 4 resource
  ignores. All 4 ignored resource tests passed explicitly, including all six
  authentic programs, prior parameter and shape mutations, all pass 8 cases,
  all seven pass 9 adversarial cases, and the cycle gate. The SmartArt
  integration filter passed 18 with 2 generator ignores and sensitivity still
  rejected a 1.01-point displacement and the calibrated pixel mutation at
  masked SSIM `0.555302336`. The three OXML projection tests passed.
  `oxml-layout --no-default-features` passed 100 tests and 3 doctests. All
  target, all-feature `rpptx` Clippy, scoped `rpptx-oxml` Clippy, formatting,
  and diff hygiene passed. These cases do not challenge D1 or D2.
