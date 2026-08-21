# S50 sprint review, pass 2

**Reviewed**: `sprint/s50` at `9fed21a` against `8f6a625e`, 25 files,
3,490 changed lines, crates: `rdocx`
**Verdict**: 2 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, excluded row blocks still skip table-level control preflight

`crates/rdocx/src/template.rs:401`
`crates/rdocx/src/template.rs:424`
`crates/rdocx/src/template.rs:754`

Pass 1 B1 is fixed for direct paragraph and row content, but not for a typed
table-level content control anchored to a row inside the block. Empty-loop and
false-condition validation invoke the row `render_item`, which renders only the
row and its nested tables. The table-level controls are handled later while
mapping emitted `Evaluated` rows. An empty loop or false condition emits no
such row, so a control containing an unclosed scalar tag or a missing root path
is discarded without inspection and the candidate commits successfully. The
fix must make row-boundary controls participate in block preflight even when
their row is excluded, while still deferring paths that genuinely depend on an
absent loop item.

### B2, repeated table-level controls bypass repeated-numbering validation

`crates/rdocx/src/template.rs:231`
`crates/rdocx/src/template.rs:310`
`crates/rdocx/src/template.rs:424`

Numbering preflight validates every table-level content control with the
table's incoming `repeated` flag before it parses the table's row blocks. For a
normal table that flag is false, so `validate_paragraph_numbering` returns
without checking a control's `numId` or level. The remediation then associates
that control with a source row and clones it once per loop iteration. A missing
numbering reference inside the repeated control can therefore reach the
committed output, contrary to the F-165 recursive preflight contract. The fix
must validate each control with the repetition state of the row block that owns
its boundary.

## Should-fix

None.

## Nice-to-have

None.

## Prior findings

- Pass 1 B1 is partially resolved. The focused
  `empty_loop_bodies_are_preflighted_without_requiring_an_item` test proves
  direct body-item validation, but B1 above leaves the table-level control path
  open.
- Pass 1 B2 is resolved for emitted rows. The focused
  `repeated_table_level_controls_use_the_row_loop_scope` test saves, reopens,
  and observes distinct values from both row scopes. B2 above is the adjacent
  numbering-preflight interaction introduced by repeating those controls.

## Milestone gate

The M16 gate is: "a template with loops, conditionals and a repeating table
row produces a correct document from a JSON data model, and every field in it
evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The complete milestone gate remains scheduled to close in S51 at
`docs/sprints/SPRINT_PLAN.md:859`. The S50 happy paths pass through
`a_nested_loop_and_conditional_generate_the_expected_document` at
`crates/rdocx/tests/regression_test.rs:3227`,
`three_template_rows_over_ten_records_produce_thirty_preserved_rows` at
`crates/rdocx/tests/regression_test.rs:3293`, and
`repeated_numbered_items_keep_one_continuous_sequence` at
`crates/rdocx/tests/regression_test.rs:3374`. The pinned Word field matrix also
passes at `crates/rdocx/tests/regression_test.rs:140`.

The S50 contribution does not yet hold across the documented structural model.
B1 still permits invalid template content to evade preflight, and B2 permits an
invalid numbering reference in repeated output. Both must be resolved before
the sprint establishes its part of the gate.

## Evidence

- `cargo test -p rdocx` passes 304 unit, integration, regression, and
  documentation tests. Two tests requiring Microsoft Word and human evidence
  remain ignored as declared.
- `python3 scripts/hash_harness.py --check` reports all 49 entries unchanged,
  matching the three AS_BUILT declarations at
  `docs/sprints/AS_BUILT.md:7557`, `docs/sprints/AS_BUILT.md:7596`, and
  `docs/sprints/AS_BUILT.md:7631`.
- `python3 scripts/prose_check.py`,
  `python3 scripts/sync_agent_skills.py --check`, and
  `git diff --check main...HEAD` pass.
- The final feature microscopes report zero remaining feature-scope defects:
  F-163 pass 2, F-164 pass 3, and F-165 pass 2.

## Not found

- `duplication`: one behavior-bearing template module owns scanning,
  structural evaluation, staging, and restoration. No duplicate sprint helper
  was introduced.
- `layering`: only the `rdocx` manifest changed. No `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: no baseline file changed. Every feature declared an unchanged or
  sprint-deferred harness, the sprint ledger declared 49 of 49 unchanged, and
  the independent check agrees.
- `docs`: the sprint contract, ledgers, three plans, and four HLD impact files
  agree on intended syntax, structural scope, native ownership, preservation,
  and the unchanged harness. B1 and B2 are implementation contradictions,
  not separate documentation drift.
- `deps`: `serde_json` is the only added crate dependency. It has the named
  `Document::render_template` consumer and already exists in the workspace
  dependency set.
- `surface`: the one added public method is required by F-163. F-164 and F-165
  extend its behavior without adding another public type or method.
- `structure`: the new template module was explicitly approved in the F-163
  design. No trait, generic parameter, forwarding wrapper, feature flag, or
  crate was added.
