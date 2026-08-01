# S16 sprint review, pass 5

**Reviewed**: `sprint/s16` at `831d23958ac50e42f67bd6a1c0c293e3093fcfa7`
against `fcfe389c71778922b7b9e5b932c4bcfb8cf97522`, 41 files, 5,501
changed lines, crates: `oxml-drawing`, `rpptx-oxml`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Review bound

The user explicitly approved extending the sprint-review loop beyond the
default three passes until the findings are resolved. This pass 5 is therefore
authorised under the command's bound rule.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Prior finding status

- **B1 is resolved.** The DrawingML helper rejects a local `xmlns:a` binding
  only when a typed parser will rewrite that element at
  `crates/oxml-drawing/src/namespace.rs:14`. List-style roots and modelled
  levels apply it before delegation at
  `crates/oxml-drawing/src/text/list_style.rs:44` and
  `crates/oxml-drawing/src/text/list_style.rs:85`. The delegated paragraph
  parser routes typed spacing and default character properties at
  `crates/oxml-drawing/src/text/paragraph.rs:915`, with checks on the spacing
  wrapper and value at `crates/oxml-drawing/src/text/paragraph.rs:182` and
  `crates/oxml-drawing/src/text/paragraph.rs:191`, character properties at
  `crates/oxml-drawing/src/text/paragraph.rs:616`, bullet elements at
  `crates/oxml-drawing/src/text/bullet.rs:708`, colour choices at
  `crates/oxml-drawing/src/color.rs:444`, and modelled empty colour transforms
  at `crates/oxml-drawing/src/color.rs:679`. The regression matrix covers
  paragraph spacing, bullets, character properties, fills, colours, and
  transforms at `crates/oxml-drawing/src/text/mod.rs:281`. The paired opaque
  tests prove that direct list-style extensions and deeper character or
  nonempty-transform captures retain their local bindings byte for byte at
  `crates/oxml-drawing/src/text/mod.rs:307` and
  `crates/oxml-drawing/src/text/mod.rs:320`. Presentation and master entry
  points both reject the original `defRPr` descendant example at
  `crates/rpptx-oxml/tests/integration.rs:616` and
  `crates/rpptx-oxml/tests/integration.rs:626`.
- **B2 remains resolved.** Slide-master boundary 6 is emitted before text
  styles at `crates/rpptx-oxml/src/slide_parts.rs:234`, with the ordering
  regression at `crates/rpptx-oxml/tests/integration.rs:566`.
- **S1 remains resolved.** Shared namespace state and fixed-prefix policy live
  once in `crates/rpptx-oxml/src/namespace.rs:23`.
- **S2 remains resolved.** The current sprint cites the implemented corpus
  contract in HLD 12 at `docs/sprints/CURRENT_SPRINT.md:20` and does not cite
  the removed HLD 13 question.

## Milestone gate

The S16 definition of done holds at the reviewed SHA. With the corpus required,
the `rpptx-oxml` integration suite reports 19 passed and zero failed across all
50 digest-verified decks. It checked 6,898 `a:txBody` elements, 8,643 `a:spPr`
elements, 50 presentation roots, 421 slides, 766 layouts, 76 masters, 1,263
shape trees, and 63 recursive groups. The opaque package comparison passed for
all 50 decks. The focused `oxml-drawing` text suite reports nine passed and zero
failed, including the typed-descendant rejection matrix and both opaque
preservation boundaries.

A fresh hash-harness check reports all 28 entries unchanged, and the pinned
corpus verifier passes. At the reviewed SHA, formatting, workspace clippy with
warnings denied, the all-feature workspace tests, the no-default-features
layout tests, the wasm check, and workspace documentation with warnings denied
all pass. Prose and generated-skill checks pass. `rpptx-oxml` remains version
0.0.0 with publication disabled at `crates/rpptx-oxml/Cargo.toml:3` and
`crates/rpptx-oxml/Cargo.toml:12`.

S16 does not close M8. The M8 end gate requires all 50 modelled decks to
round-trip and open in PowerPoint without repair at
`docs/hld/14-development-backlog.md:566`. That later manual PowerPoint
no-repair gate was not performed and is not claimed here. It remains assigned
to F-080 at `docs/hld/14-development-backlog.md:640`.

## Not found

- No interaction defect was found across F-067 through F-070. Namespace
  validation now follows every rewritten descendant reached from a typed text
  level and stops at raw subtree capture boundaries. The slide-master raw
  boundary, recursive shape tree, colour maps, presentation order, and
  identifier validation remain coherent after integration.
- No remaining namespace-helper duplication was found. DrawingML and
  PresentationML each retain one helper at their own crate boundary, while
  schema-specific dispatch remains local to each model.
- No dependency layering violation was found. `rpptx-oxml` depends toward
  `oxml-core`, `oxml-opc`, and `oxml-drawing`. No `oxml-*` crate gained an
  `rpptx-*` dependency, and the documented `oxml-drawing` to `rdocx-oxml`
  theme-adapter exception remains the only format-specific edge.
- No hash baseline changed. All 28 entries match the tracked baseline and the
  S16 ledger declares the unchanged result.
- No unnamed dependency was added. Every new edge has a present XML,
  DrawingML, or OPC consumer.
- No unrequested public surface was found. The unpublished crate, root models,
  recursive shape-tree union, namespace constants, and shared list-style type
  are required by F-067 through F-070.
- No sprint documentation contradiction was found. The backlog, current
  sprint, HLD corpus contract, architecture, AS_BUILT entries, and tracker
  agree on the completed four-story S16 slice and the unpublished development
  boundary.
