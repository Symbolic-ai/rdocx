# Current Sprint, S18

**Milestone**: M8 PresentationML.

**Goal**: Complete the long-tail PresentationML boundaries needed before the
read facade. Model connectors and notes parts, preserve alternate content while
selecting its fallback for later rendering, and make relationship identifiers
inside opaque XML safely rewritable without publishing any PowerPoint
development crate.

## Spec references

- `docs/hld/03-architecture.md`, for prefix-tolerant parsing, fixed-prefix
  writing, dependency direction, and verbatim raw XML preservation.
- `docs/hld/04-opc-and-packaging.md`, for relationship ownership, identifiers,
  and the package boundary around preserved part content.
- `docs/hld/06-presentationml-model.md`, for notes parts, the connector and
  alternate-content shape-tree arms, preservation, and relationship remapping.
- `docs/hld/12-testing-strategy.md`, for the pinned 50-deck corpus and the raw
  and modelled structural round-trip gates.
- `docs/hld/13-risks-and-open-questions.md`, for the raw-XML relationship
  remapping risk that F-078 mitigates.
- `docs/hld/14-development-backlog.md`, for the F-075 through F-078 contracts,
  dependencies, sizes, and test gates.
- `docs/hld/15-build-and-toolchain.md`, for keeping implemented `oxml-*` and
  `rpptx-*` crates at version 0.0.0 with publication disabled.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-075 | Connectors | S | pending | - |
| F-076 | mc:AlternateContent | M | pending | - |
| F-077 | Notes slides and notes master | M | pending | - |
| F-078 | relmap rewrite_rel_ids | M | pending | - |

## Sequencing note

Rows are listed in dependency order, not implementation-wave order. F-075 and
F-076 build on the completed F-070 shape tree, F-077 builds on the completed
F-069 part model, and F-078 builds on the completed F-067 XML and corpus
boundary. No S18 story depends on another S18 story. F-075 and F-076 both
change shape-tree dispatch and must be reconciled as an exclusive resource,
while F-078 remains the safety prerequisite for deep-copy work in M11.

## Definition of done for this sprint

- A typed `p:cxnSp` retains its start and end connections and every corpus
  connector round-trips structurally in shape-tree order.
- Every `mc:AlternateContent` subtree is preserved byte-identically, and its
  fallback branch is selected without changing the stored alternatives.
- Notes slides and the notes master model their required parts, expose notes
  text, and round-trip every notes-bearing corpus deck.
- Relationship remapping rewrites mapped `r:embed`, `r:link`, and `r:dm`
  identifiers inside preserved XML while leaving all other bytes unchanged.
- The relevant connector, alternate-content, notes, and preserved relationship
  payloads in all 50 pinned decks pass their structural and preservation gates.
- The full workspace gate passes with all 28 deterministic hashes unchanged,
  and no PowerPoint development crate is published.
