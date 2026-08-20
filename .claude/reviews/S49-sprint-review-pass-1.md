# S49 sprint review, pass 1

**Reviewed**: `sprint/s49` at `86feb34` against `5ca11820`, 66 files,
15,593 changed lines, crates: `oxml-core`, `oxml-opc`, `rdocx-oxml`,
`rdocx-layout`, `rdocx-html`, and `rdocx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M16 gate is: "a template with loops, conditionals and a repeating table
row produces a correct document from a JSON data model, and every field in it
evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The complete milestone gate does not hold yet, as expected after the first of
the three M16 sprints assigned at `docs/sprints/SPRINT_PLAN.md:762`. S49 supplies
the field foundation listed at `docs/sprints/SPRINT_PLAN.md:830`. Template
syntax, loops and conditionals, and repeating table rows remain the F-163,
F-164, and F-165 work described at
`docs/hld/14-development-backlog.md:1328`,
`docs/hld/14-development-backlog.md:1335`, and
`docs/hld/14-development-backlog.md:1342` and scheduled for S50 at
`docs/sprints/SPRINT_PLAN.md:846`. Their absence is therefore not an S49
defect.

The S49 contribution to that gate holds. The recursive parser corpus at
`crates/rdocx-oxml/src/text.rs:5137`, the pinned Word evaluation regression at
`crates/rdocx/tests/regression_test.rs:140`, and the update-policy and fallback
regressions at `crates/rdocx/tests/regression_test.rs:429`,
`crates/rdocx/tests/regression_test.rs:468`, and
`crates/rdocx/tests/regression_test.rs:488` pass at the reviewed SHA. Together
they cover the field and update requirements in the sprint definition at
`docs/sprints/CURRENT_SPRINT.md:45`.

## Evidence

- The final feature microscopes are clean: F-160 pass 10, F-161 pass 7, F-162
  pass 8, and F-203 pass 5 each report zero defects, smells, and nitpicks.
- Focused parser, pinned-oracle evaluation, formatting-switch, REF and PAGEREF,
  update-policy, unsupported-fallback, ordinary-save, and F-203 preservation
  tests pass at the reviewed SHA. The F-203 checks include
  `foreign_cell_width_remains_raw_and_unmodelled` at
  `crates/rdocx-oxml/src/table.rs:1957`,
  `unmodelled_standard_cell_properties_keep_absolute_slots_after_typed_mutation`
  at `crates/rdocx-oxml/src/table.rs:2122`,
  `content_control_cell_preserves_child_binding_declared_on_cell` at
  `crates/rdocx-oxml/src/content_control.rs:919`, and
  `level_raw_is_lgl_stays_before_suffix` at
  `crates/rdocx-oxml/src/numbering.rs:3167`.
- `python3 scripts/hash_harness.py --check` reports all 49 entries unchanged,
  matching the four AS_BUILT records at `docs/sprints/AS_BUILT.md:7395`,
  `docs/sprints/AS_BUILT.md:7438`, `docs/sprints/AS_BUILT.md:7481`, and
  `docs/sprints/AS_BUILT.md:7521`.
- `python3 scripts/prose_check.py`,
  `python3 scripts/sync_agent_skills.py --check`, and
  `git diff --check 5ca11820..86feb34` pass.

## Not found

- `interaction`: parser, evaluator, update policy, layout, and package-story
  traversal use the same field model and deterministic story order. Nested
  outcomes, parent and child cache updates, headers, footers, footnotes, and
  endnotes remain aligned. Zero interaction findings.
- `duplication`: no second public field model or duplicate sprint subsystem was
  introduced. Zero duplication findings.
- `layering`: no Cargo manifest changed, and no `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency. Zero layering findings.
- `harness`: the independent hash check reports 49 of 49 unchanged, with no
  baseline delta. Zero harness findings.
- `gate`: every item in the S49 definition of done at
  `docs/sprints/CURRENT_SPRINT.md:45` has executable evidence. The remaining
  template portion of the M16 gate is explicitly future sprint work. Zero S49
  gate findings.
- `docs`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, AS_BUILT, the approved
  feature plans, and their HLD impact files agree on the delivered behavior,
  intentional native Rust surface, preservation rules, tests, and unchanged
  harness. Zero documentation findings.
- `deps`: Cargo manifests and the lockfile are unchanged. Zero dependency
  findings.
- `surface`: the S49 field APIs and low-level model changes are called for by
  the approved plans. Python, WASM, and CLI surfaces remain unchanged. Public
  changes brought in by the named `origin/main` merges are not unowned S49
  feature additions. Zero surface findings.
- `structure`: no unjustified trait, generic parameter, forwarding wrapper,
  feature flag, or crate was added by the S49 stories. Zero structural
  findings.
