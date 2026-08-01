# S16 sprint review, pass 4

**Reviewed**: `sprint/s16` at `8cc2e820443156872b8a8bd4b2b82c81288a2838`
against `fcfe389c71778922b7b9e5b932c4bcfb8cf97522`, 36 files, 5,276
changed lines, crates: `oxml-drawing`, `rpptx-oxml`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Review bound

The user explicitly approved extending the sprint-review loop beyond the
default three passes until the findings are resolved. This pass 4 is therefore
authorised under the command's bound rule.

## Blocking

### B1, conflict protection stops at the level before delegated modelled children

`crates/oxml-drawing/src/text/list_style.rs:84`
`crates/oxml-drawing/src/text/list_style.rs:121`
`crates/oxml-drawing/src/text/paragraph.rs:628`
`crates/oxml-drawing/src/text/paragraph.rs:910`
`crates/oxml-drawing/src/text/paragraph.rs:704`

The pass-3 remediation correctly checks the list-style root and each direct
modelled `lvlNpPr` wrapper, while leaving an unknown direct child opaque. It
then captures a modelled level and delegates the subtree to
`CT_TextParagraphProperties`. That parser models children such as `defRPr`
without applying the fixed-prefix conflict check.

For example, a valid alternate-prefix level containing
`<d:defRPr xmlns:a="urn:producer"><a:raw/></d:defRPr>` passes because the
conflicting declaration is below the checked level start. The character
properties parser retains `xmlns:a` as a raw attribute and later writes a fixed
`a:defRPr` with that declaration. The written element is in the producer
namespace instead of DrawingML, and local-name-only reparse can still compare
equal.

Propagate the conflict check through every delegated element below a list level
that a typed parser rewrites, including paragraph spacing, bullets, character
properties, and their modelled descendants. Stop traversal as soon as a child
is classified as opaque so its local binding remains preserved. Add a focused
regression for a conflicting binding on a modelled `defRPr` or equivalent
descendant, while retaining the direct opaque-child preservation regression.

## Should-fix

None.

## Nice-to-have

None.

## Prior finding status

- **B1 remains blocking below the direct level.** The requested paired direct
  behavior is now correct. The modelled-level rejection test at
  `crates/oxml-drawing/src/text/mod.rs:275` passes, and the opaque sibling test
  at `crates/oxml-drawing/src/text/mod.rs:281` proves its complete subtree and
  local `a` binding are emitted byte for byte. The parser captures that opaque
  element as one raw subtree and does not descend. The delegated typed-child
  gap described above remains.
- **B2 remains resolved.** Slide-master boundary 6 is emitted before text
  styles at `crates/rpptx-oxml/src/slide_parts.rs:234`, with the ordering
  regression at `crates/rpptx-oxml/tests/integration.rs:566`.
- **S1 remains resolved.** Shared namespace state and fixed-prefix policy live
  once in `crates/rpptx-oxml/src/namespace.rs:26`.
- **S2 remains resolved.** The current sprint cites the implemented corpus
  contract in HLD 12 at `docs/sprints/CURRENT_SPRINT.md:20` and does not cite
  the removed HLD 13 question.

## Milestone gate

The carried M7 corpus gate holds at the reviewed SHA. With the corpus required,
the `rpptx-oxml` integration suite reports 19 passed and zero failed across all
50 digest-verified decks. It checked 6,898 `a:txBody` elements, 8,643 `a:spPr`
elements, 50 presentation roots, 421 slides, 766 layouts, 76 masters, 1,263
shape trees, and 63 recursive groups. The opaque package comparison passed for
all 50 decks. The focused `oxml-drawing` list-style tests report seven passed,
including both sides of the direct-level preservation rule.

A fresh hash-harness check reports all 28 entries unchanged, and the pinned
corpus verifier passes. At the reviewed SHA, formatting, workspace clippy with
warnings denied, the all-feature workspace tests, the no-default-features
layout tests, the wasm check, and workspace documentation with warnings denied
all pass. Prose and generated-skill checks pass. `rpptx-oxml` remains version
0.0.0 with publication disabled at `crates/rpptx-oxml/Cargo.toml:3` and
`crates/rpptx-oxml/Cargo.toml:12`.

The S16 unsupported-XML preservation definition does not fully hold while B1
remains. S16 does not close M8. The M8 end gate requires all 50 modelled decks
to round-trip and open in PowerPoint without repair at
`docs/hld/14-development-backlog.md:566`. That later manual PowerPoint
no-repair gate was not performed and is not claimed here. It remains assigned
to F-080 at `docs/hld/14-development-backlog.md:640`.

## Not found

- No other interaction defect was found across F-067 through F-070. The direct
  opaque boundary, slide-master raw boundary, recursive shape tree, colour
  maps, presentation order, and identifier validation remain coherent after
  integration.
- No remaining namespace-helper duplication was found in `rpptx-oxml`.
  Schema-specific dispatch remains local to each model.
- No dependency layering violation was found. `rpptx-oxml` depends toward
  `oxml-core`, `oxml-opc`, and `oxml-drawing`, and no `oxml-*` crate depends on
  `rpptx-oxml`.
- No hash baseline changed. All 28 entries match the tracked baseline and the
  S16 ledger declares the unchanged result.
- No unnamed dependency was added. Every new edge has a present XML,
  DrawingML, or OPC consumer.
- No unrequested public surface was found. The unpublished crate, root models,
  recursive shape-tree union, namespace constants, and shared list-style type
  are required by F-067 through F-070.
- No further sprint documentation contradiction was found. The backlog,
  current sprint, HLD corpus contract, architecture, AS_BUILT entries, and
  tracker agree on the completed four-story S16 slice and the unpublished
  development boundary.
