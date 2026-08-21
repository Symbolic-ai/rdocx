# S51 sprint review, pass 2

**Reviewed**: `sprint/s51` at `1254c0fad293af232f9a3b4896a8491faa375fae`
against merge base `cd3b34109e8d45da7d06a11d11964971c8d1568d`,
136 files and 18,011 changed lines. Crates: `oxml-chart`,
`oxml-cli-support`, `oxml-core`, `oxml-drawing`, `oxml-layout`,
`oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx-layout`,
`rdocx-oxml`, `rdocx-wasm`, `rdocx`, `rpptx-chart`, `rpptx-cli`,
`rpptx-layout`, `rpptx-oxml`, `rpptx-render`, `rpptx-wasm`, and `rpptx`

**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Prior finding

Pass 1 B1 is resolved. The staging clone still starts with empty derived caches
at `crates/rdocx/src/document.rs:476`. On successful commit,
`commit_staged_mutation` swaps the live normal-engine mutex into the candidate
before replacing the document at `crates/rdocx/src/document.rs:482`. Failed
watermark staging returns before that commit, so the live engine and completed
results remain untouched. Successful text and image setters both use this
boundary at `crates/rdocx/src/document.rs:1626` and
`crates/rdocx/src/document.rs:1661`.

The focused regression populates normal and deterministic completed results,
exercises both setters, requires the completed caches to clear while the normal
engine remains present, and requires a distinct post-mutation accepted result
at `crates/rdocx/src/document.rs:8180`. The retained engine's paragraph cache
behavior is independently locked by
`warm_relayout_matches_cold_and_rebuilds_only_changed_safe_paragraphs` at
`crates/rdocx-layout/src/engine.rs:3623`. Together the code and tests establish
that unchanged safe body and shaping work remains available without allowing a
stale completed result.

## Milestone gate

The M16 gate is: "a template with loops, conditionals and a repeating table row
produces a correct document from a JSON data model, and every field in it
evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The gate holds at the reviewed SHA. The parser corpus at
`crates/rdocx-oxml/src/text.rs:5139`, the pinned Word field matrix at
`crates/rdocx/tests/regression_test.rs:140`, the nested loop and conditional
fixture at `crates/rdocx/tests/regression_test.rs:3323`, and the three-row by
ten-record fixture at `crates/rdocx/tests/regression_test.rs:3389` pass. The
complete `rdocx` suite also passes 154 unit tests, 92 integration tests, 120
regression tests, and two documentation tests. The two ignored tests retain
their declared Microsoft Word and human-evidence requirements.

## Evidence

- The pass-1 remediation commit changes only the pass-1 review artifact and
  `crates/rdocx/src/document.rs`. The public surface, dependency graph, package
  state, HLD, release notes, contributor record, and baselines are unchanged.
- The new watermark regression, the F-X038 warm-layout and diagnostic-replay
  regressions, the complete `rdocx` suite, and the field parser corpus pass at
  `1254c0fad293af232f9a3b4896a8491faa375fae`.
- `python3 scripts/hash_harness.py --check` reports all 49 entries unchanged.
  No hash or golden baseline file differs from the merge base, consistent with
  the completed feature records at `docs/sprints/AS_BUILT.md:7667` through
  `docs/sprints/AS_BUILT.md:7939`.
- `cargo metadata --no-deps` reports 27 workspace packages and no forbidden
  `oxml-*` dependency on an `rdocx-*` or `rpptx-*` crate. The manifest and lock
  delta adds no external dependency and remains confined to the approved 0.4.0
  incubating carriers and internal pins beginning at `Cargo.toml:55`.
- The `rpptx-v0.4.0` notes still pass deterministic validation. The incubating
  metadata, exact dependency-ordered publication routing, and rendered-notes
  workflow regressions pass. The reviewed notes retain their exact scope and
  contributor evidence at `CHANGELOG.md:103` and `CHANGELOG.md:142`.
- PR 36 still retains Pedro Assumpcao's original commit as the second parent of
  merge commit `92951e71474383b48ce7ede194be4d0f34729488`. The delivery record
  preserves the contribution and current-base CI evidence at
  `docs/sprints/AS_BUILT.md:7913` and `docs/sprints/AS_BUILT.md:7933`.
- `python3 scripts/prose_check.py`,
  `python3 scripts/sync_agent_skills.py --check`, metadata layering inspection,
  and `git diff --check` pass.

The recorded full verification in `.claude/scratch/S51-run.json:146` still
names the pre-remediation SHA. `/release` therefore still requires a fresh full
verification record at this reviewed SHA before any release approval or
external mutation, as required at `.claude/commands/release.md:57`. This is the
next process gate, not a sprint-review finding.

## Not found

- `interaction`: watermark staging now preserves the F-X038 engine while
  invalidating F-X032 completed result caches. Mail merge, comparison,
  provenance, ordered-body access, and release preparation retain their
  reviewed ownership boundaries. No remaining cross-feature defect was found.
- `duplication`: the one private staged-commit helper owns the exceptional
  whole-document watermark commit. Template and comparison continue to commit
  only their changed typed state. No duplicate sprint subsystem was found.
- `layering`: no forbidden crate edge was added.
- `harness`: every declaration agrees with the independent 49-entry check and
  no baseline changed.
- `gate`: the exact M16 end gate has direct passing evidence above.
- `docs`: the plans, HLD, sprint contract, and delivery record agree on
  persistent-engine ownership, atomic watermark staging, public surfaces,
  package preservation, and release boundaries.
- `deps`: no external dependency was added.
- `surface`: the remediation adds no public API. Every integrated public type,
  field, and function remains owned by an approved S51 story.
- `structure`: the private staged-commit helper has two present callers and
  owns the non-forwarding engine-transfer operation. No unowned trait, generic,
  feature flag, crate, module, file, or forwarding wrapper was added.
