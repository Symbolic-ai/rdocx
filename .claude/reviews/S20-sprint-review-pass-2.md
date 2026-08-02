# S20 sprint review, pass 2

**Reviewed**: `sprint/s20` against `31a0249d50a767f43c99eb53af0436143825d56d`, 35 files, 3,508 changed lines, crates: `oxml-drawing`, `rpptx-oxml`, `rpptx-layout`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M9 end gate at `docs/hld/14-development-backlog.md:653` is: "the contract
is frozen and published to the render track."

S20 does not complete M9, so this gate is not yet due and is not claimed. The
S20 slice holds. All five named feature tests pass, as do the full workspace
gate, the normal-stack 50-deck structural corpus test, the exact 40-case colour
table, and all 28 unchanged deterministic hashes. F-086 through F-088 remain
pending and own the flattener, frozen `ResolvedSlide`, and final differential
evidence.

## Not found

- Interaction: B1 is fixed. The raw scanners resolve namespace bindings at
  `crates/oxml-drawing/src/effect.rs:399`, and the two regressions at
  `crates/rpptx-layout/src/style.rs:531` and
  `crates/rpptx-layout/src/style.rs:545` prove that foreign `effectDag` and
  `schemeClr` elements cannot alter DrawingML resolution.
- Duplication: each inheritance concern and raw-effect classification has one
  implementation.
- Layering: `rpptx-layout` depends downward on `rpptx-oxml` and
  `oxml-drawing`. No `oxml-*` crate gained an `rpptx-*` dependency.
- Harness: every AS_BUILT entry reports the observed unchanged 28-entry result,
  including the post-remediation check at `docs/sprints/AS_BUILT.md:2491`.
- Gate: each S20 story has focused tests, and the integrated parser, corpus,
  colour, dependency, version, and publication riders passed.
- Docs: the design amendment and AS_BUILT entry record the namespace finding,
  its two regressions, and the unchanged intended resolver contract.
- Deps: `rpptx-layout` has only its two approved model dependencies and no new
  third-party dependency.
- Surface: the sprint exposes only the approved concrete resolver and typed
  model surfaces. Every PowerPoint crate remains `0.0.0` and unpublished.
