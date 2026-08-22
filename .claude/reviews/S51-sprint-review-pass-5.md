# S51 sprint review, pass 5

**Reviewed**: `sprint/s51` at `b98d3aa57cb7fec73a82a546e941e3cd59ee5970`
against merge base `cd3b34109e8d45da7d06a11d11964971c8d1568d`,
152 files and 19,058 changed lines. Crates: `oxml-chart`,
`oxml-cli-support`, `oxml-core`, `oxml-drawing`, `oxml-layout`,
`oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx`, `rdocx-cli`,
`rdocx-html`, `rdocx-layout`, `rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`,
`rdocx-py`, `rdocx-wasm`, `rpptx`, `rpptx-chart`, `rpptx-cli`,
`rpptx-layout`, `rpptx-oxml`, `rpptx-py`, `rpptx-render`, and `rpptx-wasm`

**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

The integrator explicitly authorized this fifth pass solely for the mandatory
post-`/release` F-X036 finalization required by `/run-sprint` step 9.0. The
release was approved and completed only after the explicitly bounded clean
pass 4. Publication then required the release F-ID's deferred plan, delivery
ledger, sprint state, and current-state HLD updates before sprint closure.

This review records the explicit extension decision required by the bounded
review rule at `.claude/commands/sprint-review.md:45`. Its scope is limited to
the 11-file, 109-line finalization delta from the reviewed release tag to the
current HEAD. That delta contains the F-X036 plan completion, the four delivery
records, current-state HLD 03, 10, 12, and 15, the agent-facing current-version
statement, and its matching workflow assertion. It does not change product
code, manifests, the lockfile, release notes, release workflows, or published
tag content.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Release evidence

- The annotated local and remote `v0.8.0` tag both peel to reviewed SHA
  `0cc47eb8632de184ba758fe0929d9f749ab4fcb0`, matching the durable record at
  `docs/sprints/AS_BUILT.md:7994` and the current release boundary at
  `docs/hld/15-build-and-toolchain.md:266`.
- GitHub workflow run 32536705662 completed successfully at that exact SHA.
  Its output-stability, metadata, reviewed-notes, archive, stable publication,
  and GitHub release steps passed. The incubating publication step was skipped,
  as the AS_BUILT record states at `docs/sprints/AS_BUILT.md:7995`.
- All seven selected crates resolve from crates.io at 0.8.0, and every owner
  endpoint reports `mantissaman`. All seven rendered README endpoints are
  non-empty, which supplies the post-publication evidence described at
  `docs/hld/12-testing-strategy.md:644`.
- The non-draft, non-prerelease GitHub release is attached to `v0.8.0`. Its
  complete 9,291-character body is byte-identical to a fresh
  `release-notes v0.8.0 --render`, matching
  `docs/sprints/AS_BUILT.md:7998`. Contributor credit for Pedro Assumpcao,
  Issue 37, and Issue 39 remains present in the published body.
- The post-release commit changes none of `Cargo.toml`, `Cargo.lock`,
  `CHANGELOG.md`, `.github/`, `.claude/commands/`, or `crates/`. Published
  artifacts and their reviewed notes therefore remain anchored to the release
  tag rather than the later ledger commit.

## Tracker and state consistency

F-X036 is completed in its design plan at
`.claude/plans/F-X036-design.md:3`, in the active sprint at
`docs/sprints/CURRENT_SPRINT.md:43`, and in the backlog at
`docs/sprints/BACKLOG.md:439`. Its one-day completion row appears exactly once
at `docs/sprints/SPRINT_TRACKER.md:288`, and its one durable completion record
begins at `docs/sprints/AS_BUILT.md:7983`.

Run state also records F-X036 completed with no owner at
`.claude/scratch/S51-run.json:91`. `close-preflight` found no feature, owner,
tracker, handoff, or ledger inconsistency. Its only refusals are the expected
need for a clean review and full verification at the final post-review HEAD.

The completed plan's deferred publication checklist now matches the release
evidence at `.claude/plans/F-X036-design.md:140`. Its HLD impact list names
exactly the four HLD files changed by finalization at
`.claude/plans/F-X036-design.md:97`.

## HLD current intent

- HLD 03 now states that the exact seven-package stable family is published at
  0.8.0 from the annotated tag while retaining separate approval for later
  releases at `docs/hld/03-architecture.md:407`.
- HLD 10 states the shipped low-level 0.8 compatibility boundary and the
  unchanged Python, WASM, and CLI surfaces at
  `docs/hld/10-bindings-spec.md:334`. It also truthfully identifies provenance
  as published in both the incubating and stable families at
  `docs/hld/10-bindings-spec.md:343`.
- HLD 12 changes prepared-state wording to the current 0.8.0 release and keeps
  the README checks assigned to the post-publication gate at
  `docs/hld/12-testing-strategy.md:644`.
- HLD 15 names the exact reviewed tag SHA, exact stable family, and unchanged
  binding and incubating authority at `docs/hld/15-build-and-toolchain.md:260`.
  It retains the separate approval and exact-SHA contract for every later
  release at `docs/hld/15-build-and-toolchain.md:284`.
- `CLAUDE.md` now gives agents the current crates.io version and exact family at
  `CLAUDE.md:14`. The repository-claim regression derives that statement from
  the workspace version at `scripts/test_sprint_workflow.py:5847` and retains a
  mutation that rejects reintroducing prepared-state wording at
  `scripts/test_sprint_workflow.py:5946`.

These are current-state replacements, not appended change history. They follow
the design plan's exact HLD impact list and preserve the HLD ownership and
precedence boundaries.

## Issue and pull request communication

GitHub Issue 37 has a stable-release follow-up that names the four complete
layout accessors, their cached and caller-font ownership, complete font data,
diagnostics, and result-local provenance. The issue is closed. PR 36 has a
stable-release follow-up that names `Document::body_items`, confirms Pedro
Assumpcao's original commit and merge record were preserved, and thanks the
contributor. The durable summary records both communications at
`docs/sprints/AS_BUILT.md:8024`.

Issue 38 has its shipped provenance follow-up and is closed. Issue 39 has its
shipped cache follow-up and a later maintainer response that distinguishes the
released exact typed paragraph key from two unshipped proposals. The public
`FontData` payload remains `Vec<u8>` at
`crates/oxml-layout/src/output.rs:299`, and `LayoutResult.pages` remains
`Vec<PageFrame>` at `crates/oxml-layout/src/output.rs:345`. No public engine
take or set method exists.

The newest Issue 39 pagination proposal was posted after the v0.8.0 workflow
and GitHub release completed. It proposes restartable pagination, shared page
frames, table caching, and separate footnote-reference invalidation work. None
of those changes appears in the immutable release tag or the current product
tree, and the published notes make no claim that they shipped. Keeping that
new public type and cache work outside v0.8.0 is therefore correct. The
AS_BUILT handoff already directs later Issue 39 proposals to separate
post-release review at `docs/sprints/AS_BUILT.md:8024`.

## Milestone gate

The M16 gate remains: "a template with loops, conditionals and a repeating
table row produces a correct document from a JSON data model, and every field
in it evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The finalization delta changes no product code, fixture, or release artifact.
The recorded full gate at the release SHA remains valid for the published
family, and independent focused checks at the current HEAD report all 66
workflow tests passing and all 49 deterministic hash entries unchanged. The
M16 product evidence reviewed in pass 4 is unaffected.

## Closure boundary

After this artifact is committed, the integrator must run the required final
exact-HEAD verification and record this clean pass with the explicit extension
flag. At inspection time the latest recorded clean review and full verification
still cover the release tag commit at `.claude/scratch/S51-run.json:149` and
`.claude/scratch/S51-run.json:190`. That is the expected pre-review state, not a
defect in the finalization delta. Sprint closure remains forbidden until those
two exact-HEAD predicates pass.

## Not found

- `interaction`: the finalization delta has no runtime or published-artifact
  interaction with any S51 feature.
- `duplication`: one plan, one completion record, one tracker row, and the
  existing agent-facing assertion remain authoritative.
- `layering`: no crate or dependency edge changed.
- `harness`: all 49 deterministic entries remain unchanged.
- `gate`: the M16 product gate and both release-family gates retain direct
  evidence.
- `docs`: the plan, HLD, active sprint, backlog, tracker, AS_BUILT record,
  current-version statement, and run state agree.
- `deps`: no manifest, lockfile, version, allowlist, or dependency changed.
- `surface`: no public product or binding surface changed. The new Issue 39
  proposals remain explicitly outside the released surface.
- `structure`: no product construct, module, file, feature flag, or wrapper was
  added by finalization.
