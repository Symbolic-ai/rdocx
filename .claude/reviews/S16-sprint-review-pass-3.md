# S16 sprint review, pass 3

**Reviewed**: `sprint/s16` at `22117ea403ae83750064cf10e063aeb457653f49`
against `fcfe389c71778922b7b9e5b932c4bcfb8cf97522`, 35 files, 5,155
changed lines, crates: `oxml-drawing`, `rpptx-oxml`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the list-style conflict scan rejects opaque XML it does not rewrite

`crates/oxml-drawing/src/text/list_style.rs:36`
`crates/oxml-drawing/src/text/list_style.rs:119`
`crates/oxml-drawing/src/text/list_style.rs:209`
`docs/hld/06-presentationml-model.md:94`

The pass-2 remediation closes the original corruption path by scanning the
whole list-style subtree before parsing. The scan examines every start and
empty element and rejects every non-DrawingML `xmlns:a` declaration. It does
not distinguish modelled elements from the unknown children that
`capture_child` stores as opaque raw XML.

A valid subtree such as
`<x:extension xmlns:x="urn:extension" xmlns:a="urn:producer"><a:data/></x:extension>`
is never rewritten. It would retain its local binding and namespace meaning if
captured and emitted verbatim. The new pre-scan rejects it before the raw-child
path can preserve it. This changes the shared DrawingML parser from accepting
and preserving unsupported XML to rejecting it, contrary to the sprint goal
and the HLD preservation contract.

Restrict conflict rejection to elements that this parser or a delegated typed
parser actually rewrites. Do not descend through an opaque child once its raw
boundary is selected. Add a regression in which an unknown list-style child
locally binds `a` to a producer namespace and round-trips byte for byte, beside
the existing regression that rejects the same binding on a modelled level.

## Should-fix

None.

## Nice-to-have

None.

## Prior finding status

- **B1 remains blocking in a new failure mode.** The original namespace
  corruption is closed. The focused cases at
  `crates/rpptx-oxml/tests/integration.rs:611` and
  `crates/rpptx-oxml/tests/integration.rs:616` reject a conflicting `a` binding
  below presentation default text style and master title style. Master title,
  body, and other styles all use the same indexed branch and parser call at
  `crates/rpptx-oxml/src/slide_parts.rs:833` and
  `crates/rpptx-oxml/src/slide_parts.rs:848`. The whole-subtree implementation
  introduces the opaque-child rejection described above.
- **B2 remains resolved.** Slide-master boundary 6 is emitted before text
  styles at `crates/rpptx-oxml/src/slide_parts.rs:234`, with the ordering
  regression at `crates/rpptx-oxml/tests/integration.rs:566`.
- **S1 remains resolved.** Shared namespace state and fixed-prefix policy live
  once in `crates/rpptx-oxml/src/namespace.rs:26`.
- **S2 remains resolved.** The live sprint references the implemented corpus
  contract in HLD 12 at `docs/sprints/CURRENT_SPRINT.md:20` and does not cite
  the removed HLD 13 question.

## Review bound

This is pass 3 of the configured maximum of 3. Because one blocking finding
remains, the sprint-review exit condition is not met and S16 is not ready to
close. The workflow does not permit a fourth pass without an explicit decision
to extend the bound. B1 must therefore be carried or the bound must be extended
explicitly after remediation.

## Milestone gate

The carried M7 corpus gate holds at the reviewed SHA. With the corpus required,
the `rpptx-oxml` integration suite reports 19 passed and zero failed across all
50 digest-verified decks. It checked 6,898 `a:txBody` elements, 8,643 `a:spPr`
elements, 50 presentation roots, 421 slides, 766 layouts, 76 masters, 1,263
shape trees, and 63 recursive groups. The opaque package comparison passed for
all 50 decks. The focused `oxml-drawing` namespace regression also passes.

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
`docs/hld/14-development-backlog.md:566`. That manual PowerPoint no-repair gate
was not performed and is not claimed here. It remains assigned to F-080 at
`docs/hld/14-development-backlog.md:640`.

## Not found

- No other interaction defect was found across F-067 through F-070. The
  original nested binding cases, slide-master raw boundary, recursive shape
  tree, colour maps, presentation order, and identifier validation remain
  coherent after integration.
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
