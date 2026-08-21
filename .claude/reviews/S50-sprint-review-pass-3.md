# S50 sprint review, pass 3

**Reviewed**: `sprint/s50` at `8244b43` against `8f6a625e`, 26 files,
3,765 changed lines, crates: `rdocx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Prior findings

- Pass 1 B1 and pass 2 B1 are resolved. Empty loops and false conditions now
  evaluate excluded items with validation-only sentinels at
  `crates/rdocx/src/template.rs:787`. Deferred lookup continues through
  concrete outer scopes and the root before deferring an item-dependent path at
  `crates/rdocx/src/template.rs:1032`. The focused regression covers malformed
  scalar syntax, missing root paths, nested missing loop paths, and a concrete
  outer `settings.missing` path under an empty inner loop at
  `crates/rdocx/src/document.rs:6405`. Excluded table-level controls now travel
  with their row item through the same validation path at
  `crates/rdocx/src/template.rs:433`, with both empty-loop and false-condition
  cases at `crates/rdocx/src/document.rs:6554`.
- Pass 1 B2 is resolved. Every emitted row owns its table-level controls during
  evaluation at `crates/rdocx/src/template.rs:403`, so cloned controls use the
  emitted row's lexical scope. The save and reopen regression observes distinct
  values for two iterations at `crates/rdocx/src/document.rs:6473`.
- Pass 2 B2 is resolved. Numbering validation associates each table-level
  control with the parsed row block and propagates the loop repetition state at
  `crates/rdocx/src/template.rs:249`. The missing repeated control `numId`
  regression is at `crates/rdocx/src/document.rs:6580`.
- Preservation ordering remains intact. Raw siblings are copied from each
  source boundary to each emitted row boundary, while every control retains its
  raw-children-before ordinal at `crates/rdocx/src/template.rs:460`. Trailing raw
  siblings and controls remain at the reconstructed trailing boundary at
  `crates/rdocx/src/template.rs:476`.

## Milestone gate

The M16 gate is: "a template with loops, conditionals and a repeating table
row produces a correct document from a JSON data model, and every field in it
evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The complete milestone gate remains scheduled to close in S51 at
`docs/sprints/SPRINT_PLAN.md:859`. The S50 contribution now holds. The nested
loop and conditional regression passes at
`crates/rdocx/tests/regression_test.rs:3227`, the three-row by ten-record gate
passes at `crates/rdocx/tests/regression_test.rs:3293`, continuous numbering
passes at `crates/rdocx/tests/regression_test.rs:3374`, and the pinned Word field
matrix passes at `crates/rdocx/tests/regression_test.rs:140`.

## Evidence

- `cargo test -p rdocx` passes 306 unit, integration, regression, and
  documentation tests. Two tests requiring Microsoft Word and human evidence
  remain ignored as declared.
- `cargo clippy -p rdocx --all-targets -- -D warnings` passes.
- `python3 scripts/hash_harness.py --check` reports all 49 entries unchanged,
  matching the three AS_BUILT declarations at
  `docs/sprints/AS_BUILT.md:7557`, `docs/sprints/AS_BUILT.md:7596`, and
  `docs/sprints/AS_BUILT.md:7631`.
- `python3 scripts/prose_check.py`,
  `python3 scripts/sync_agent_skills.py --check`, and
  `git diff --check main...HEAD` pass.
- The sprint state records a successful full verification over the integrated
  feature result. The later review remediations are confined to
  `crates/rdocx/src/document.rs` and `crates/rdocx/src/template.rs`, and the
  scoped crate, clippy, harness, prose, adapter, and diff checks pass at the
  reviewed head.

## Not found

- `interaction`: no remaining conflict between scalar staging, structural
  exclusion, lexical scopes, row repetition, content controls, numbering, or
  preserved boundaries. All four earlier blocking paths are resolved above.
- `duplication`: one behavior-bearing template module owns scanning,
  structural evaluation, scalar staging, restoration, and preflight. No second
  sprint helper implements the same behavior.
- `layering`: only the `rdocx` manifest changed. No `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- `harness`: no baseline file changed. Every feature records an unchanged
  harness, and the independent check reports 49 of 49 unchanged.
- `gate`: the S50 portion of the M16 gate has direct passing regressions. The
  remaining milestone closure work is correctly assigned to S51.
- `docs`: the sprint contract, ledgers, three plans, and four listed HLD impact
  files agree on syntax, structural scope, native facade ownership, package
  preservation, and unchanged binding surfaces.
- `deps`: `serde_json` is the only added crate dependency at
  `crates/rdocx/Cargo.toml:38`. It already exists in the workspace and has the
  named public `Document::render_template` consumer at
  `crates/rdocx/src/document.rs:2576`.
- `surface`: the one added public method is required by F-163. F-164 and F-165
  extend its behavior without another public type or method.
- `structure`: the focused template module at
  `crates/rdocx/src/lib.rs:34` was explicitly approved in the F-163 design. No
  trait, generic parameter, forwarding wrapper, feature flag, or crate was
  added.
