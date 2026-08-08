# S27 sprint review, pass 1

**Reviewed**: `sprint/s27` at `f3de77e` against merge base `ba9b6ed`,
32 files, 4,237 changed lines (4,143 additions and 94 deletions), crates:
`oxml-drawing`, `rpptx-oxml`, and `rpptx`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, completed feature owners use an invalid empty sentinel

`docs/sprints/CURRENT_SPRINT.md:39`
`scripts/sprint_workflow.py:209`

All four completed S27 rows leave `Owner` empty at
`docs/sprints/CURRENT_SPRINT.md:39` through line 42. The workflow accepts only
`-` for a completed feature and reports every empty value as a durable-record
problem. `close-preflight S27` therefore refuses closure for F-109 through
F-112. Replace each empty completed owner cell with the canonical `-` sentinel
and establish that close preflight no longer reports these four mismatches.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M11 end-of-milestone gate is: "a generated 10-slide deck opens clean in
PowerPoint, Keynote, Google Slides and LibreOffice."

S27 does not complete M11, so the four-viewer 10-slide gate is not yet due and
was not claimed. The S27 shapes-and-text gate itself holds. Every shape setter
round-trips, all four shape constructors validate and opened without repair in
Microsoft PowerPoint 16.104 build 16.104.25121423, native picture sizing matches
pinned python-pptx 1.0.2, and placeholder text replacement round-trips and
changes the deterministic render. The integrated full workspace verification
passed with all 28 hash entries unchanged. Every changed PowerPoint development
crate remains unpublished at version 0.0.0.

## Not found

- Interaction: F-110 construction and F-111 picture insertion both reuse the
  F-109 shape-tree allocation and preservation rules, while F-112 text handles
  remain confined to ordinary shapes. Their combined append, mutation, save,
  reopen, validation, and render coverage shows no conflicting ownership or
  lost state.
- Duplication: shape and picture construction share the facade transform helper,
  every constructor shares `ShapeIdAllocator` and `CT_ShapeTree::append_child`,
  and picture insertion retains the existing `MediaStore`. No competing helper
  path was added.
- Layering: the new `rpptx` edges point to lower-level `oxml-core` and
  `oxml-drawing`. No changed `oxml-*` manifest gained an `rdocx-*` or
  `rpptx-*` dependency.
- Harness: no baseline or harness file changed. All four plans and completion
  records declare an unchanged harness, and integrated verification matched all
  28 entries.
- Gate: the focused round-trip, preservation, schema-order, package,
  deterministic-render, oracle, and native PowerPoint checks cover the S27
  definition of done. The later M11 cross-viewer gate remains assigned to S28.
- Dependencies: `oxml-core` supplies the public `Emu`, `Angle`, and facade error
  boundary, while `oxml-drawing` supplies the planned fill, line, transform,
  geometry, and text types. Both new direct dependencies have current consumers.
- Surface: public additions match the four approved plans. No unrequested
  trait, generic, wrapper, crate, module, feature flag, or constructor family
  was introduced.
