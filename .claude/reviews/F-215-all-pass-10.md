# F-215, all, pass 10

**Reviewed**: complete working diff against the F-215 worker base, 17 files,
4,913 additions and 89 deletions, plus the completed design, all six HLD impact
files, progress notes, passes 1 through 9, and every default microscope aspect
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the timing gate does not enforce the two parameter categories the HLD claims

`docs/hld/12-testing-strategy.md:65`

`crates/rpptx-oxml/tests/integration.rs:889`

`crates/rpptx-oxml/tests/integration.rs:947`

`crates/rpptx-oxml/tests/integration.rs:977`

The updated HLD says the timing corpus gate requires nonzero coverage for both
transition parameters and effect parameters. The test combines slide-transition
effect parameters and timing `p:animEffect` parameters into one
`effect_parameters` counter, then makes one nonzero assertion on their sum. A
regression that drops either category remains green while the other category is
present. The completion prose therefore overstates the independent coverage
that the executable gate provides.

### D2, the testing strategy claims a cross-deck regression that does not exist

`docs/hld/12-testing-strategy.md:84`

`crates/rpptx/tests/integration.rs:1450`

`crates/rpptx/src/lib.rs:1784`

The new media-gate prose says source-built cases cover cross-deck
shape-reference remapping. The related package regression creates one
presentation and calls `duplicate_slide` inside that same presentation. The
facade exposes that same-deck duplication operation, not a cross-deck slide
copy or import operation. The HLD records coverage beyond the implemented and
tested F-215 surface, contrary to the current-intent rule for completion
updates.

## Smells

None.

## Nitpicks

None.

## Completion-gate follow-up

- The correction removes only the requirement that the current corpus contain
  at least one unsupported timing node. The counter and raw-byte inventory
  remain active (`crates/rpptx-oxml/tests/integration.rs:900` and
  `crates/rpptx-oxml/tests/integration.rs:985`).
- Complete timing and transition raw XML is still compared before and after
  serialization (`crates/rpptx-oxml/tests/integration.rs:923`), and unsupported
  node byte inventories are still compared exactly
  (`crates/rpptx-oxml/tests/integration.rs:934`). Slide, layout, and master
  structural equality checks also remain in the corpus walk
  (`crates/rpptx-oxml/tests/integration.rs:842`).
- The source-built unsupported-node regression still retains its exact
  `p:animClr` subtree while allocating timing ids around it
  (`crates/rpptx-oxml/tests/integration.rs:242`). The complete 50-deck timing
  corpus gate passes with zero unsupported nodes, and the focused regression
  passes.
- The completed plan lists six HLD impact files, and exactly those six HLD
  files changed (`.claude/plans/F-215-design.md:197`). The scope, ownership,
  package, PresentationML, and native API updates describe current implemented
  behavior. The testing-strategy overclaims are recorded as D1 and D2.

## Not found

- Correctness beyond the findings: no new production-code regression was found
  in media inspection, atomic mutation, signature checks, timing ownership,
  shape-reference remapping, relationship retention, or candidate-only part
  pruning.
- Contract beyond the findings: the implementation checklist matches the
  completed F-215 surface. Required posters, exact external targets, opaque
  unsupported payloads, replacement invariants, and native-only scope remain
  consistent between code and the other five HLD updates.
- Panics: the added production indexing and `expect` sites remain dominated by
  checked slide indices, parser-established roots, or fixed local
  construction.
- OOXML: namespace-aware direct ownership, exact Office extension paths,
  schema child order, lexical trim preservation, raw relationship retention,
  and modelled-slot-only timing removal remain intact.
- Tests beyond D1 and D2: the complete timing corpus gate passes across 50
  decks, the source-built unsupported-node regression passes, and the embedded
  audio and video package corpus gate passes against both pinned decks.
- Structure: no new trait, generic, feature, crate, module, dependency,
  forwarding wrapper, or builder was introduced. The six HLD changes are
  exactly the design plan's declared completion work list.
