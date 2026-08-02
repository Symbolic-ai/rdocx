# S18 sprint review, pass 1

**Reviewed**: sprint/s18 against b163c2155cd6, 24 files, 3,699 changed
lines, crates: oxml-drawing, rpptx-oxml
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M8 end gate is: "all 50 corpus decks round-trip, and every one opens in
PowerPoint without a repair prompt."

The integrated `all_corpus_decks_round_trip_opaquely` and structural corpus
tests passed for all 50 pinned decks. The PowerPoint repair-prompt gate was not
performed in S18 and is not claimed complete. It remains assigned to F-080,
which is still pending in the M8 backlog.

The S18 slice gate holds. The connector corpus test covered 85 connectors,
including 6 nested connectors. The alternate-content corpus test preserved all
21 discovered subtrees byte-identically. The notes tests covered 210 notes
slides, 24 notes masters, 72 nonempty note bodies, and relationship
cardinality. The preserved-payload test proved empty-map identity across all 50
decks, while the focused relationship test rewrote `r:embed`, `r:link`, and
`r:dm` without changing surrounding bytes. The integrated full workspace gate,
all risk riders, and all 28 deterministic hashes passed. Manifest inspection
confirmed every `oxml-*` and `rpptx-*` crate remains version 0.0.0 with
publication disabled.

## Not found

- Interaction: no conflict between connector dispatch, alternate-content
  fallback dispatch, typed shape text, recursive groups, or notes extraction.
- Duplication: no second shape-tree dispatcher, text extractor, or relationship
  parser was introduced under another name.
- Layering: no manifest changed. The existing `rpptx-oxml` dependency direction
  remains intact, and no `oxml-*` crate gained an `rpptx-*` dependency.
- Harness: no output delta. Every AS_BUILT entry records the unchanged 28-entry
  result.
- Gate: no vacuous S18 story gate. Focused fixtures and nonzero corpus coverage
  exercise each delivered boundary.
- Docs: no implementation contradiction remains outside the approved HLD
  impact. The optional `p:notesStyle` schema correction and fallback-selection
  contract are reflected in the current HLD.
- Deps: no dependency or feature-flag change was introduced.
- Surface: no public API outside the four approved story contracts was added.
