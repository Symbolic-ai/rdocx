# F-162, all, pass 7

**Reviewed**: complete working tree against `HEAD` (`6a60586`), 7 files, 2,587 additions and 65 deletions, excluding review artifacts
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, removing a multi-run stale descendant collapses its boundary runs

`crates/rdocx-oxml/src/text.rs:2669`

`NestedComplexSpan.start` and `end` are field-marker offsets inside the start
and end runs. When the stale descendant spans multiple runs, draining that
whole byte range also removes the closing tag of the start run, every complete
intermediate run, and the opening tag of the end run. Content before the begin
marker and after the end marker is consequently merged under the start run's
wrapper. The saved XML can silently lose the end run's properties and producer
attributes. It can also become namespace-invalid when surviving suffix content
uses a prefix declared only on the removed end-run start tag, which makes the
staged update fail. The pass-6 regression places the outer, middle, and stale
grandchild in one physical run at `crates/rdocx/src/field.rs:2301`, so it does
not exercise this multi-run branch. Raw-wins removal needs to delete the typed
descendant while retaining the physical run scaffolding around its marker
span.

## Smells

None.

## Nitpicks

None.

## Pass-6 repair verification

- D1 is closed for the exact same-run middle-field reproduction. Before save,
  evaluation reports only the outer and effective middle fields in preorder.
  The update writes both surviving caches, clears both dirty flags, removes the
  stale instruction and display, preserves adjacent producer XML in order, and
  reopens with the same two evaluations and values.
- The new raw-wins branch removes every immediate typed descendant before it
  rewrites the effective instruction, so the prior source-count mismatch no
  longer occurs for a same-run descendant.

## Checks

- `cargo test -p rdocx-oxml text::tests`, passed, 70 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 16 tests.
- `cargo test -p rdocx --test regression_test`, passed, 79 tests.
- `cargo clippy -p rdocx -p rdocx-oxml --all-targets --no-deps -- -D warnings`,
  passed.
- `cargo fmt --all -- --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/sync_agent_skills.py --check`, passed, 25 skills in sync.
- Progress evidence for full crate tests, package dry-run, archive size, and the
  remaining repository gates was inspected.

## Not found

No additional defect was found in same-run raw-wins descendant removal,
preorder update identity, shared-boundary sibling replacement, hyperlink
trivia, opaque source exclusion, package paragraph span mapping, parsed source
identity, F-161 story traversal, simple-field mutation, cache and dirty policy,
atomic live-state commit, layout invalidation, update-aware save delegation,
leave-alone save APIs, settings and property preservation, schema child order,
binding scope, panic safety outside D1, HLD scope, tests, or structure. No
smells or nitpicks were found.
