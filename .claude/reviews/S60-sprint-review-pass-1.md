# S60 sprint review, pass 1

**Reviewed**: sprint/s60 against ece49e64fd8e3ea94ec7eb47a417a0742e522cec, 23 files, 5,142 lines, crates: rpptx-oxml, rpptx
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold at this F-213 dependency-prefix boundary. F-214
is still pending at `docs/sprints/CURRENT_SPRINT.md:32`, so explicit timestamp
evaluation, transition rendering, animated output, and their pinned PowerPoint
oracle evidence are not present. Later M21 stories still own SmartArt, media,
package variants, animated export, notes export, and handout export. This is
expected at the prefix boundary and does not block F-214 from starting.

The F-213 dependency gate does hold. The story gate at
`docs/hld/14-development-backlog.md:1905` requires the corpus timeline to parse
into the declared model, serialize in schema order, and preserve every
unsupported sibling. The exact test
`the_corpus_timeline_preserves_every_unsupported_sibling` at
`crates/rpptx-oxml/tests/integration.rs:476` passed independently at reviewed
HEAD e074178c433b04c857e2724e2f505e796354cb57. It verified all 50 pinned decks
and observed 421 slides, 766 layouts, 76 masters, 272 timing roots, 372
transitions, 732 typed nodes, 401 conditions, 25 builds, 82 set values, 186
effect parameters, 10 compatibility transitions, and 5 unsupported nodes.
These counters exactly match the completion record at
`docs/sprints/AS_BUILT.md:10305`. The test compares retained timing and
transition source bytes before and after serialization at
`crates/rpptx-oxml/tests/integration.rs:586`, compares unsupported node
inventories at `crates/rpptx-oxml/tests/integration.rs:608`, and structurally
compares every parsed root with its reparse at
`crates/rpptx-oxml/tests/integration.rs:501`.

The ordinary static-output condition also holds at this boundary. The F-213
plan declares an unchanged harness at `.claude/plans/F-213-design.md:156`, the
completion record reports 49 of 49 unchanged at
`docs/sprints/AS_BUILT.md:10314`, and this review reproduced all 49 entries at
the pinned head. The sprint run state records full integrated verification at
the same head. This review reran only the exact corpus gate and hash harness,
not the broad verification suite.

## Not found

Interaction produced zero findings. F-213 is the only implemented story in the
reviewed prefix. Its typed model is integrated and verified before F-214, which
matches the strict sequencing contract at
`docs/sprints/CURRENT_SPRINT.md:38`. The approved F-214 evaluator inputs are
present: common-node identity and duration at
`crates/rpptx-oxml/src/timing.rs:100`, condition state at
`crates/rpptx-oxml/src/timing.rs:93`, container state at
`crates/rpptx-oxml/src/timing.rs:115`, behavior targets and values at
`crates/rpptx-oxml/src/timing.rs:129`, and transition policy at
`crates/rpptx-oxml/src/timing.rs:320`.

Duplication produced zero findings. The timing parser, compatibility selection,
raw-byte mutation, and root integration each have one implementation in the
approved timing module and existing slide-parts module.

Layering produced zero findings. The sprint changes no Cargo manifest, and no
`oxml-*` crate gained a dependency on an `rdocx-*` or `rpptx-*` crate. Timing
evaluation remains absent from `rpptx-oxml`, consistent with the completion
boundary at `docs/sprints/AS_BUILT.md:10316`.

Harness produced zero findings. The plan and completion record both declare
the unchanged result, and the exact reviewed head reproduced 49 of 49 entries.

Gate produced zero prefix-scope findings. The named corpus test exercises
nonzero coverage for every bounded category required by the testing contract
at `docs/hld/12-testing-strategy.md:63`, while source-built mutation and package
tests cover the approved lexical and save-reopen paths.

Docs produced zero findings. The changed HLD files are exactly the F-213 design
plan impact list at `.claude/plans/F-213-design.md:134`. Their preservation,
native API, and corpus-gate descriptions agree with the integrated source and
tests.

Dependencies produced zero findings. No manifest changed and the new module
uses existing workspace dependencies only.

Surface produced zero findings. The additive public timing and transition
values are the low-level pre-1.0 API called for by F-213. No facade execution
API, renderer behavior, binding method, CLI option, trait, generic, feature,
crate, or production dependency was added.
