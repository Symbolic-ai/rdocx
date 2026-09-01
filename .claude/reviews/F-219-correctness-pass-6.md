# F-219, correctness, pass 6

**Reviewed**: the complete updated working-tree implementation and plan diff, ten files with 3,533 added lines and 34 removed lines including the untracked diagram module, plus the five prior reviews with 382 lines
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, complete SmartArt relationship validation is still structural rather than semantic
`crates/rpptx/src/lib.rs:3374`

Transfer preflight walks the relationships that happen to exist, but never
compares a SmartArt frame's four `dgm:relIds` values or its data model's
`dsp:dataModelExt/@relId` with the required relationship types. A missing
`r:qs` id can therefore survive the rewrite unchanged, and a drawing id can
name an image or another permitted relationship type. The setter has the same
gap because it type-checks only the data relationship at
`crates/rpptx/src/lib.rs:2398`, then relies on generic package validation at
`crates/rpptx/src/lib.rs:2422`. Generic validation does not type-check the
layout, style, colour, and drawing roles, and it does not discover the
unqualified drawing id when its relationship is absent. These malformed graphs
can be transferred or edited successfully instead of failing atomically under
the plan's complete-graph contract. The current failure test removes a target
part, which generic validation detects, but does not exercise a missing id or a
wrong relationship type in any of the other four roles.

### D2, transfer does not require exactly one internal source layout relationship
`crates/rpptx/src/lib.rs:3374`

The preflight loop rejects an external slide-layout relationship but never
counts internal slide-layout relationships. With none, the caller's selected
destination layout is never added. With two, both relationships are retargeted
to the selected layout by `crates/rpptx/src/lib.rs:3493`, leaving two layout
relationships. `commit_candidate` writes and reopens without calling
`validate()` at `crates/rpptx/src/lib.rs:1775`, so either invalid slide can be
published even though `Presentation::validate()` requires exactly one layout
relationship. This bypasses the explicit destination-layout ownership
contract. The external-layout regression does not cover missing or duplicate
internal layouts.

### D3, duplicate diagram point identities make a checked edit ambiguous
`crates/rpptx-oxml/src/diagram.rs:621`

The point-list parser appends every typed point without checking whether its
`modelId` is already present. `set_node_text` then edits the first matching
point at `crates/rpptx-oxml/src/diagram.rs:369` and serializes both duplicates.
An externally supplied data model with two editable points named `n1` therefore
reports success after changing an arbitrary one, rather than rejecting the
ambiguous identity. The approved test plan explicitly requires a duplicate-id
fixture, but the added round-trip tests cover ordering and duplicate root
children only, not duplicate point or connection identities.

### D4, typed projections accept same-namespace lookalikes from opaque extension positions
`crates/rpptx-oxml/src/diagram.rs:452`

Layout title, algorithm, constraint, and category extraction scans every
descendant by expanded name without checking its schema-owned path. Style and
colour labels do the same at `crates/rpptx-oxml/src/diagram.rs:507` and
`crates/rpptx-oxml/src/diagram.rs:550`. The cached drawing id is likewise taken
from the first `dsp:dataModelExt` anywhere in the data part at
`crates/rpptx-oxml/src/diagram.rs:256`. A producer-owned wrapper under an
extension list can therefore contain a same-namespace `dgm:alg` that changes an
otherwise unsupported layout into `Pyramid`, or a nested `dsp:dataModelExt`
that is treated as the drawing owner and rewritten during duplication. Such
content should remain opaque unless it occupies the schema-owned position.
The namespace tests use foreign-namespace lookalikes, so they do not detect
this same-namespace path confusion.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-5 remediation: complete serialized-slide placeholder scanning now covers GraphicFrame raw non-visual XML and preserved compatibility branches.
- Transfer closure: slide-owned and diagram-owned images with relationships reject before staging. Unsupported internal diagram dependencies also reject.
- Cycles and depth: preflight and copy share the 128-part ceiling, terminate cycles, and no longer expose unbounded recursive depth.
- Layout ownership: an external source layout rejects and a valid single source layout is retargeted to the explicitly selected destination layout. The remaining zero-or-multiple case is D2.
- Collision handling: copied diagram parts use destination-visible fresh names, graph cycles reuse their allocated copies, and image bytes use the existing media deduplication path.
- OOXML preservation: inherited relationship namespaces, fixed `dgm`, `a`, and `r` shadows, safe text-root attributes, ordered data-model children, raw direct events, and complete-root parsing remain corrected. The remaining schema-position issue is D4.
- Producing scopes: slide, layout, and master SmartArt are inspected against their own relationship scopes. Duplicate and transfer paths remap relationship-namespace attributes and the schema-recognized cached drawing id.
- Atomicity: tested rejection paths stage changes before publication. No additional partial-publication defect was found beyond operations that incorrectly succeed in D1 and D2.
- Panics: no reachable unchecked index, arithmetic overflow, unbounded graph recursion, or malformed canonical-text slice was found.
- Structure: the approved diagram module and five-instantiation `DiagramPart<T>` generic satisfy the repository structural rules. No unjustified trait, dynamic dispatch, dependency, feature, crate, wrapper, or integration binary was added.
- Tests: the focused SmartArt integration selections passed 7 `rpptx-oxml` tests and 11 `rpptx` tests, and `git diff --check` passed. The isolated worktree has no corpus directory, so the corpus-dependent tests skipped their external data. The sensitivity gaps are recorded in D1 through D4.
