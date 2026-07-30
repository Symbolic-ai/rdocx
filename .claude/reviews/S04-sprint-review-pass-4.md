# S04 sprint review, pass 4

**Reviewed**: `sprint/s04` at
`cc137193e742b7990a98abc2a9c22407bf0fcd7e` against
`f464f756f5d425683d7a1c83173c84418e4c1011`, 32 files, 2,982 changed lines,
crates: `oxml-opc`
**Bound extension**: The user explicitly authorized a fourth pass after pass 3
reached the default bound. The sprint run state records
`max_review_passes = 4` at `.claude/scratch/S04-run.json:74`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Prior findings

Pass 3 B1 is resolved. The independently assembled package now uses the valid
slide-layout identifier `2147483648` at
`crates/oxml-opc/src/package.rs:402`, and the test locks the PresentationML
minimum at `crates/oxml-opc/src/package.rs:627`. The presentation uses slide ID
256 and master ID 2147483648 at `crates/oxml-opc/src/package.rs:383`. Its
relationship graph reaches presentation to master and slide, slide to layout,
layout to master, and master to layout and theme through
`crates/oxml-opc/src/package.rs:363` to
`crates/oxml-opc/src/package.rs:380`. All referenced parts have content-type
overrides at `crates/oxml-opc/src/package.rs:349` to
`crates/oxml-opc/src/package.rs:358`.

Pass 3 S1 is resolved. The migration plan describes a version-independent
breaking cutover at `docs/hld/11-migration-plan.md:156`, and records the actual
seven-package allowlist plus the post-PowerPoint F-049 expansion at
`docs/hld/11-migration-plan.md:169`. The current workflow still names exactly
`rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`, `rdocx-pdf`, `rdocx`,
and `rdocx-cli` at `.github/workflows/publish.yml:23` through
`.github/workflows/publish.yml:60`.

## Milestone gate

The M2 gate is: hash harness unchanged, and `OpcPackage` opens a real `.pptx`
in a test.

The gate holds. `python3 scripts/hash_harness.py --check` matched all 28
entries, and `scripts/hash_baseline.json` has no sprint diff. All 19
`oxml-opc` tests pass, including
`independently_built_pptx_opens_and_resolves_relationships`. That test opens a
ZIP assembled independently of `OpcPackage::write_to`, checks the complete
minimum presentation, slide, layout, master, and theme graph, and verifies that
every target resolves to a present part at
`crates/oxml-opc/src/package.rs:552` through
`crates/oxml-opc/src/package.rs:633`.

The development publication boundary also holds. `oxml-opc` remains version
0.0.0 with `publish = false` at `crates/oxml-opc/Cargo.toml:3` and
`crates/oxml-opc/Cargo.toml:10`. No released rdocx manifest or publication
workflow is changed by the sprint. The roadmap defers publication readiness to
S32.1 at `docs/sprints/SPRINT_PLAN.md:463`, requires separate reviewed release
approval before S32.2 at `docs/sprints/SPRINT_PLAN.md:475`, and defers every
released-rdocx consumer cutover to S32.2 at
`docs/sprints/SPRINT_PLAN.md:479`. Current metadata reports exactly the seven
released rdocx packages as publishable, with `oxml-core`, `oxml-opc`, and
`rdocx-wasm` unpublished.

## Sprint state

The sprint run state and delivery records agree. F-018 through F-021 are
completed, while F-015, F-016, and F-022 are carried at
`.claude/scratch/S04-run.json:2` through `.claude/scratch/S04-run.json:87` and
`docs/sprints/CURRENT_SPRINT.md:31` through
`docs/sprints/CURRENT_SPRINT.md:39`. The carried stories target S32.2 in
`docs/sprints/BACKLOG.md:64` through `docs/sprints/BACKLOG.md:71`. All four
completed worker handoffs are consumed, their integration commits are recorded,
and their retained worktrees are clean.

## Not found

- **Interaction**: no jointly incorrect behaviour was found across F-018
  through F-021.
- **Duplication**: the copied OPC implementation is the approved isolated
  staging copy, bounded by the deferred F-022 cutover.
- **Layering**: `cargo tree -p oxml-opc --edges normal` contains only
  `quick-xml`, `thiserror`, and `zip` as direct dependencies, with no
  `rdocx-*` or `rpptx*` edge.
- **Harness**: no baseline edit or undeclared output delta was found.
- **Gate**: no remaining package-graph, relationship-target, content-type, or
  identifier-range defect was found in the independent PresentationML fixture.
- **Docs**: no remaining conflict was found among the migration HLD, backlog,
  sprint roadmap, current sprint contract, and AS_BUILT record.
- **Deps**: each new dependency has a direct named consumer in `oxml-opc`.
- **Surface**: no public API beyond the generic constructors and constant
  modules required by F-018 and F-019 was found.
