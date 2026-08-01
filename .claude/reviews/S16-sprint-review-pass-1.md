# S16 sprint review, pass 1

**Reviewed**: `sprint/s16` against `fcfe389c71778922b7b9e5b932c4bcfb8cf97522`, 32 files, 4,944 changed lines, crates: `oxml-drawing`, `rpptx-oxml`
**Verdict**: 2 blocking, 2 should-fix, 0 nice-to-have

## Blocking

### B1, nested modelled elements can rebind fixed writer prefixes

`crates/rpptx-oxml/src/slide_parts.rs:617`
`crates/rpptx-oxml/src/shape_tree.rs:319`

F-068 rejects conflicting `p`, `a`, and `r` bindings at every element that it
rewrites with fixed prefixes. The F-069 and F-070 parsers do not carry that
rule through their nested modelled boundaries. A colour-map choice can be
recognised through an alternate DrawingML prefix while locally binding `a` to
a producer namespace. A required `p:nvGrpSpPr` or `p:grpSpPr` can do the same
with `p`, and a typed group transform can do it with `a`. These elements are
then rewritten with canonical prefixes. Their canonical declarations are
either removed by the raw-attribute filter at
`crates/rpptx-oxml/src/slide_parts.rs:1082` and
`crates/rpptx-oxml/src/shape_tree.rs:649`, or retained on a newly canonical
wrapper, so preserved descendants change namespace or the modelled wrapper is
written in the producer namespace. Apply the canonical-prefix conflict check
at every nested element that becomes modelled, including colour-map choices,
master text styles, required group shells, and group transforms. Add focused
regressions in which an alternate valid element prefix coexists with a locally
conflicting fixed prefix used by preserved raw XML.

### B2, slide-master boundary six is written on the wrong side of text styles

`crates/rpptx-oxml/src/slide_parts.rs:229`
`crates/rpptx-oxml/src/slide_parts.rs:235`

For a slide master, the parser advances from `p:hf` to boundary 6 and reserves
boundary 7 for `p:extLst` at
`crates/rpptx-oxml/src/slide_parts.rs:124`. Boundary 6 therefore represents
unsupported XML after `p:hf` and before `p:txStyles`. The writer emits only
boundaries 2 through 5, writes `p:txStyles`, and then emits boundary 6. Any
producer extension in that slot moves across the modelled text styles. The
first parse and reparse also place it in different `OrderedRawChildren` slots.
Emit boundary 6 before text styles and add a structural round-trip regression
with raw XML between `p:hf` and `p:txStyles`.

## Should-fix

### S1, namespace state and fixed-prefix policy are implemented three times

`crates/rpptx-oxml/src/presentation.rs:24`
`crates/rpptx-oxml/src/slide_parts.rs:1030`
`crates/rpptx-oxml/src/shape_tree.rs:585`

Three private `NamespaceBindings` implementations duplicate prefix resolution,
attribute decoding, canonical-conflict checks, and raw namespace filtering.
They have already diverged in exactly the invariant behind B1. Consolidate the
shared namespace state and fixed-prefix policy in the existing
`namespace.rs`, while leaving schema dispatch local to each model file.

### S2, the current sprint cites a corpus question that completion removed

`docs/sprints/CURRENT_SPRINT.md:22`
`docs/hld/13-risks-and-open-questions.md:28`

The current sprint says HLD 13 carries the requirement to settle the deck
corpus source, but F-067 removed Q3 after settling it and HLD 13 now proceeds
directly from Q2 to the risk list. Point the live sprint reference at the
implemented corpus contract in HLD 12, or remove the stale HLD 13 entry.

## Nice-to-have

None.

## Milestone gate

The carried M7 gate holds. With the fetched corpus required, the complete
`rpptx-oxml` integration suite reports 18 passed and zero failed across 50
digest-verified decks. The test at
`crates/rpptx-oxml/tests/integration.rs:248` checked 6,898 `a:txBody` and 8,643
`a:spPr` elements. The opaque package gate at
`crates/rpptx-oxml/tests/integration.rs:179` passed for all 50 decks. The
presentation, part, and tree gates covered 50 presentation roots, 421 slides,
766 layouts, 76 masters, 1,263 shape trees, and 63 recursive groups. A fresh
hash-harness run reports all 28 entries unchanged, the corpus verifier passes,
and `rpptx-oxml` remains version 0.0.0 with publication disabled at
`crates/rpptx-oxml/Cargo.toml:3` and
`crates/rpptx-oxml/Cargo.toml:11`.

The S16 preservation definition does not fully hold until B1 and B2 are fixed.
S16 does not close M8. The later M8 end gate requires every modelled corpus
deck to round-trip and open in PowerPoint without repair at
`docs/hld/14-development-backlog.md:566`. That manual PowerPoint no-repair gate
was not performed for S16 and is not claimed here. F-080 remains responsible
for it.

## Not found

- No other interaction defect was found across the four stories. F-067 lands
  before model work, F-070 replaces the intended F-069 raw shape-tree boundary,
  and F-068 retains slide order for the later part models.
- No hash baseline changed. Commit messages defer worker checks to the
  integrated gate, the sprint ledger declares the unchanged result, and a
  fresh check confirms all 28 entries.
- No dependency layering violation was found. `rpptx-oxml` depends toward
  `oxml-core`, `oxml-opc`, and `oxml-drawing`, while no `oxml-*` crate depends
  on `rpptx-oxml`.
- No unnamed dependency was added. Every new crate edge has a present parser,
  package, DrawingML model, or XML reader consumer.
- No unrequested public API was found. The new crate, root models, shape-tree
  union, namespace constants, constructors, and list-style wrapper entry points
  are called for by F-067 through F-070.
- No corpus fetch or manifest defect was found. Check mode validates exactly 50
  files, rejects missing and extra entries, and verifies every pinned digest.
