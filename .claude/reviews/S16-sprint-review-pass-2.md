# S16 sprint review, pass 2

**Reviewed**: `sprint/s16` at `ee0ee2331ce5309d65aad23b9926c2c6cfb9208d`
against `fcfe389c71778922b7b9e5b932c4bcfb8cf97522`, 33 files, 4,991
changed lines, crates: `oxml-drawing`, `rpptx-oxml`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, fixed-prefix protection stops above modelled list-style descendants

`crates/rpptx-oxml/src/presentation.rs:188`
`crates/rpptx-oxml/src/slide_parts.rs:844`
`crates/oxml-drawing/src/text/list_style.rs:80`
`crates/oxml-drawing/src/text/list_style.rs:112`
`crates/oxml-drawing/src/text/paragraph.rs:853`
`crates/oxml-drawing/src/text/paragraph.rs:935`

The pass-1 remediation now rejects fixed-prefix conflicts on the
`p:defaultTextStyle`, `p:titleStyle`, `p:bodyStyle`, and `p:otherStyle`
wrappers, but both call sites then hand the complete subtree to
`CT_TextListStyle`. That parser recognises each level by local name alone and
models it without namespace bindings. A valid alternate-prefix child such as
`d:lvl1pPr xmlns:a="urn:producer"` therefore passes the wrapper check.
`CT_TextParagraphProperties` retains that local `xmlns:a` as a raw attribute,
then writes a fixed `a:lvl1pPr` and reattaches the declaration. The written
level element is consequently in `urn:producer`, not DrawingML. Reparse can
still compare equal because the same parser again uses only the local name.

Carry namespace bindings and fixed-prefix conflict protection through every
list-style descendant that becomes modelled, or reject the subtree before it
enters the DrawingML parser. Add focused regressions for a nested level under
both presentation default text style and master text style. The regression
must prove that an alternate DrawingML element prefix plus a locally conflicting
`a` binding is rejected or written without changing namespace meaning.

## Should-fix

None.

## Nice-to-have

None.

## Pass-1 finding status

- **B1 remains blocking.** The focused remediation test at
  `crates/rpptx-oxml/tests/integration.rs:600` covers colour-map choices,
  PresentationML text-style wrappers, required group shells, group transforms,
  and recursive groups. It does not cover the modelled DrawingML levels below
  a text-style wrapper.
- **B2 is resolved.** Slide-master raw boundary 6 is emitted at
  `crates/rpptx-oxml/src/slide_parts.rs:234`, before text styles are written at
  `crates/rpptx-oxml/src/slide_parts.rs:235`. The structural ordering regression
  places and checks producer XML before `p:txStyles` at
  `crates/rpptx-oxml/tests/integration.rs:566`.
- **S1 is resolved.** Namespace state, attribute decoding, canonical URI
  policy, conflict rejection, and fixed declaration filtering now live in the
  existing `crates/rpptx-oxml/src/namespace.rs:26`. The three model modules use
  that shared implementation.
- **S2 is resolved.** The live sprint references the implemented corpus
  contract in HLD 12 at `docs/sprints/CURRENT_SPRINT.md:18` and no longer cites
  the removed HLD 13 question.

## Milestone gate

The carried M7 corpus gate holds at the reviewed SHA. With the fetched corpus
required, the `rpptx-oxml` integration suite reports 19 passed and zero failed
across all 50 digest-verified decks. It checked 6,898 `a:txBody` elements,
8,643 `a:spPr` elements, 50 presentation roots, 421 slides, 766 layouts, 76
masters, 1,263 shape trees, and 63 recursive groups. The opaque part comparison
also passed for every deck. A fresh hash-harness check reports all 28 entries
unchanged, and the pinned corpus verifier passes.

At the current SHA, formatting, workspace clippy with warnings denied, the
all-feature workspace tests, the no-default-features layout tests, the wasm
check, and workspace documentation with warnings denied all pass. Prose and
generated-skill checks also pass. `rpptx-oxml` remains version 0.0.0 with
publication disabled at `crates/rpptx-oxml/Cargo.toml:3` and
`crates/rpptx-oxml/Cargo.toml:12`.

The S16 preservation definition does not fully hold while B1 remains. S16 does
not close M8. The M8 end gate requires all 50 modelled decks to round-trip and
open in PowerPoint without repair at
`docs/hld/14-development-backlog.md:566`. That manual PowerPoint no-repair gate
was not performed for S16 and is not claimed here. F-080 remains responsible
for it.

## Not found

- No additional interaction defect was found across F-067 through F-070 beyond
  B1. The slide-master boundary fix does not move the later extension boundary,
  and the consolidated namespace state preserves each model's schema dispatch.
- No remaining duplicate namespace implementation was found. Schema-specific
  parsing stays in its owning model while shared namespace mechanics live once
  in `namespace.rs`.
- No dependency layering violation was found. `rpptx-oxml` depends toward
  `oxml-core`, `oxml-opc`, and `oxml-drawing`, and no `oxml-*` crate depends on
  `rpptx-oxml`.
- No hash baseline changed. A fresh check confirms all 28 entries and the S16
  ledger declares the unchanged result.
- No unnamed dependency was added. Each new edge has a present XML, DrawingML,
  or OPC consumer.
- No unrequested public surface was found. The unpublished crate, root models,
  recursive shape-tree union, and namespace constants are required by F-067
  through F-070.
- No further sprint documentation contradiction was found. The backlog,
  current sprint, HLD corpus contract, architecture, AS_BUILT entries, and
  tracker agree on the completed four-story S16 slice and the unpublished
  development boundary.
