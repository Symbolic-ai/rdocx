# S50 sprint review, pass 1

**Reviewed**: `sprint/s50` at `6dbce3c` against `8f6a625e`, 24 files,
3,231 changed lines, crates: `rdocx`
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, empty loops skip scalar and path preflight

`crates/rdocx/src/template.rs:720`

The loop body is evaluated only inside the iteration over resolved array
values. When that array is empty, the body is never passed to `render_item` and
none of its scalar tags or nested lookup paths are inspected. A template with
an empty `items` array can therefore contain an unclosed scalar tag, a missing
root path, or a missing nested loop path inside `{% for item in items %}` and
still succeed. The candidate removes the whole block and commits, even though
the public method promises that malformed tags and missing paths fail without
mutation. This is the same preflight class that the explicit false-condition
validation handles, but empty loops bypass it. The fix must validate loop-body
syntax and every path that can be resolved independently of an absent loop
item, while retaining valid empty-loop behavior for item-dependent paths.

### B2, repeated table-level controls are evaluated outside their loop scope

`crates/rdocx/src/template.rs:394`
`crates/rdocx/src/template.rs:417`

Row-loop evaluation passes each direct row through `render_item` with the
iteration scope, but table-level `w:sdt` controls are copied afterward as
boundary sidecars. A typed row control immediately before a repeated source
row is cloned once per iteration, yet scalar tags inside that control are left
untouched. The later document-wide scalar pass has only the root JSON scope,
so `{{ item.name }}` in every cloned control either fails as missing or renders
the same root `item` value instead of the current iteration value. This is an
F-164 and F-165 interaction defect in the content-control preservation claim.
The fix must evaluate each cloned table-level control under the same lexical
scope as the repeated row while preserving its raw-boundary position.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M16 gate is: "a template with loops, conditionals and a repeating table
row produces a correct document from a JSON data model, and every field in it
evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The complete milestone gate is scheduled to close in S51 at
`docs/sprints/SPRINT_PLAN.md:859`. The S50 happy-path contribution has direct
evidence: `a_nested_loop_and_conditional_generate_the_expected_document` at
`crates/rdocx/tests/regression_test.rs:3227`,
`three_template_rows_over_ten_records_produce_thirty_preserved_rows` at
`crates/rdocx/tests/regression_test.rs:3293`, and
`repeated_numbered_items_keep_one_continuous_sequence` at
`crates/rdocx/tests/regression_test.rs:3374` all pass. The pinned Word field
evidence from S49 also passes at
`crates/rdocx/tests/regression_test.rs:140`.

The S50 contribution does not yet hold for the documented structural model.
B1 permits invalid template content to evade preflight when a loop is empty,
and B2 breaks lexical evaluation for typed row controls repeated with table
rows. Both must be resolved before the sprint can establish its part of the
gate.

## Evidence

- `cargo test -p rdocx` passes all 302 nonignored unit, integration,
  regression, and documentation tests. Two tests requiring Microsoft Word and
  human evidence remain ignored as declared.
- `python3 scripts/hash_harness.py --check` reports all 49 entries unchanged,
  matching the three AS_BUILT declarations at
  `docs/sprints/AS_BUILT.md:7557`,
  `docs/sprints/AS_BUILT.md:7596`, and
  `docs/sprints/AS_BUILT.md:7631`.
- `git diff --check main...HEAD` passes.
- The final feature microscopes report zero remaining feature-scope defects:
  F-163 pass 2, F-164 pass 3, and F-165 pass 2.

## Not found

- `duplication`: one behavior-bearing template module owns scanning,
  structural evaluation, and scalar staging. No duplicate sprint helper was
  introduced.
- `layering`: only `rdocx` manifests changed. No `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: no baseline file changed, every feature declared an unchanged
  harness, and the independent check reports 49 of 49 unchanged.
- `docs`: CURRENT_SPRINT, BACKLOG, SPRINT_TRACKER, AS_BUILT, the three approved
  plans, and all four HLD impact files agree on scope, delivery state, native
  API ownership, and the unchanged harness.
- `deps`: `serde_json` is the only added crate dependency. It has the named
  `Document::render_template` consumer and already exists in the workspace
  dependency set.
- `surface`: the one added public method is required by F-163. F-164 and F-165
  extend its behavior without adding another public type or method.
- `structure`: the new template module was explicitly approved in the F-163
  design. No trait, generic parameter, forwarding wrapper, feature flag, or
  crate was added.
