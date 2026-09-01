# F-220, correctness, pass 4

**Reviewed**: claim base `3d87d827351508b38ffb22103f26351c3f18c0ca`
through exact source head `445dcda7b4bed91883ac60ac86210d7e0c065c8d`,
6 files and 3,930 changed lines, with 3,830 additions and 100 deletions. The
review includes the full pass-3 remediation, approved revised plan, cited HLD
sections, progress record, three prior untracked reviews, and complete tracked
and untracked state.
**Verdict**: 5 defects, 0 smells, 0 nitpicks. Not ready because the defects
remain and the external PowerPoint completion evidence is absent.

## Defects

### D1, presentation-point text overrides the authoritative data-node text
`crates/rpptx/src/diagram.rs:495`

The render node selects `point.text` before `owner.text`. The approved topology
contract makes the data node reached through `presOf` authoritative. If both a
presentation point and its owning data point carry text, the presentation copy
therefore renders and a checked F-219 edit to the data node remains invisible.
The existing ownership regression supplies text only on the data point, so it
does not distinguish the reversed precedence.

### D2, an unmapped ordinary data node is silently omitted
`crates/rpptx/src/diagram.rs:481`

Ownership validation requires every presentation point to have one data owner,
but it never requires every ordinary or assistant data point to own a
presentation point. A model with two ordinary data nodes, one presentation
node, and one valid `presOf` mapping therefore renders as a supported one-node
diagram while silently dropping the second data node. The supported unscoped
program has no selector that could exclude it. This is missing ownership and
must fail closed rather than produce plausible incomplete output.

### D3, sub-unit literal constraints are reinterpreted as scale factors
`crates/rpptx/src/diagram.rs:1046`

The parser correctly distinguishes `val` from `fact`, and `apply_constraints`
passes that distinction into `resize_axis`. The resize then treats a literal
`val` at or below 1 as a fraction of the complete frame extent. For example,
`type="w" val="0.5"` in a 240-point frame requests 120 points instead of the
literal 0.5 points. A `fact="0.5"` is the scale-factor form and should remain
the only path that multiplies an existing extent. This magnitude-dependent
reinterpretation violates the approved literal-or-scale contract and can
return supported but wrong geometry.

### D4, the transient SmartArt group has no authoritative bounds clip
`crates/rpptx/src/diagram.rs:1273`

`apply_frame_transform` copies offset, extent, child coordinates, rotation, and
flips, but creates no clip carrier. PresentationML group bounds do not clip
their children, so text, strokes, and effects can paint outside the original
graphic frame even when every generated node rectangle is contained. The
six-family unit calls `validate_rects` at
`crates/rpptx/src/diagram.rs:1626` and treats that containment check as the
clipping assertion. It never inspects a resolved or rendered group clip. This
does not satisfy the contract that the complete diagram group is clipped to
its bounds.

### D5, the required constraint, style, and colour unit contract is absent
`crates/rpptx/src/diagram.rs:2027`

The private unit suite has a rejection test for targeted constraints, rules,
and spacing, but no successful test for literal `val`, scale `fact`, multiple
constraints in schema order, or their interaction with resolved quick styles
and transformed colours. The exact approved unit
`smartart_constraints_rules_styles_and_colours_resolve_in_declared_order` is
absent. The current approach intentionally rejects every rule, while the test
plan at `.claude/plans/F-220-design.md:218` still requires supported rule
precedence. The contract and test must be reconciled, and the remaining
supported constraint, style, colour, and text behavior needs one discriminating
shared-engine test. The missing literal-versus-factor case is why D3 remains
green.

## Smells

None.

## Nitpicks

None.

## External completion evidence

The six-family PowerPoint oracle remains absent. Ordinary mode explicitly
skips at `crates/rpptx/tests/integration.rs:269` when
`f220-smartart-oracle.tsv` is missing. Required-corpus mode fails at that exact
path before comparing a family. The progress record states the same external
blocker at `.claude/scratch/F-220-progress.md:100` and does not claim feature
completion. Under the differential-testing policy, this is an external
feature-completion blocker, not a defect, smell, or nitpick. The green
source-built geometry and Rust raster tests do not establish the approved
PowerPoint 16.104, 1-point, and 0.99 SSIM acceptance gate.

## Not found

- **Pass-3 D1, exact family mapping**: Layout family inference now requires the
  declared exact identity and algorithm pair. Relationship plus `cycle` is
  distinct from Cycle plus `cycle`, and undeclared substring matches remain
  unsupported.
- **Pass-3 D2, parameter allowlists**: Unknown and duplicate projected
  parameters, invalid list direction, malformed or out-of-range matrix columns,
  and family-disallowed parameters fail closed. List reversal no longer depends
  on aspect ratio. D3 is the remaining supported constraint-value semantic
  defect, not a recurrence of guessed algorithm parameters.
- **Pass-3 D3 and D4, topology ordinals and graph validity**: Both typed
  connection ordinals survive projection and normalize stable node and edge
  order. Duplicate semantic edges and cycles reject before every family layout.
  D1 and D2 concern the adjacent ownership mapping, not ordinal loss or
  family-specific cycle validation.
- **Pass-3 D5, public API**: `DiagramRenderProjection` again has exactly the
  approved `algorithms`, `constraints`, and `rules` fields. Style ownership
  travels in the algorithm record, and the scoped resource types remain private
  to `rpptx`. No facade, renderer, Python, WASM, or CLI API was added.
- **Pass-3 D6, animation coverage**: The producing-scope integration now
  exports a deterministic GIF before and after the F-219 text edit, requires
  changed animation bytes, and rejects an unsupported-SmartArt diagnostic.
  Static, timeline, and media assertions remain in the same test.
- **Pass-3 D7, family geometry**: The private suite now pins exact rectangles
  for all six families, node order, connector endpoints and ordinals,
  relationship cardinality, finite extents, and frame transform identity. D4 is
  limited to the still-unproven and unimplemented outer clip.
- **OOXML and preservation**: The doc-hidden projections retain their approved
  shape, traverse bounded schema-owned expanded-name paths, reject aliases
  shadowed to foreign namespaces, exclude extension lookalikes, retain colour
  transform order, and keep source XML byte exact.
- **Panics and numeric bounds**: No reachable untrusted-input panic or unchecked
  arithmetic overflow was found. Graph counts and depth, layout depth, finite
  geometry, containment, and EMU conversion remain bounded or fail closed.
- **Structure and dependency direction**: The one new private module is
  approved. No new trait, generic, feature, crate, dependency, dynamic dispatch,
  integration binary, or binary fixture was added. Parsing remains in
  `rpptx-oxml`, expansion remains in `rpptx`, and `rpptx-render` remains
  unchanged.
- **Focused validation**: `cargo test -p rpptx --lib diagram::tests` passed 10
  tests. The three focused `rpptx-oxml` render-projection tests passed.
  `cargo test -p rpptx --test integration smartart_ -- --nocapture` passed 18
  tests with one ignored generator and the documented missing-oracle skip.
  `cargo check -p rpptx --all-targets` and `git diff --check` passed. Required
  corpus mode failed at the absent manifest as designed.
