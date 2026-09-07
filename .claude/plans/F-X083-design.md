# F-X083, Close confirmed Issue 67 and intake Issue 69

**Status**: completed
**Sprint**: S70
**Size**: S
**Depends on**: F-X075, F-X076

## Problem

Issue 67 remains open even though the shipped F-X075 implementation and its
deterministic regression preserve completed pagination across page-spanning
prose. The regression proves that 175 four-line paragraphs publish restart
state only at complete boundaries, then that ten sourced warm edits retain 174
paragraph hits, rebuild one paragraph, paginate at most two pages, and equal a
fresh deterministic result (`crates/rdocx-layout/src/engine.rs:11813` and
`crates/rdocx-layout/src/engine.rs:11858`). The HLD records the same
complete-boundary contract (`docs/hld/08-rendering-spec.md:791`).

The reporter has confirmed that v0.12.0 fixed Issue 67 and explicitly said it
can be closed. Issue 69 remains open with three independent performance
mechanisms. Current main still disables paragraph-cache reads from one complete
retained-context result, recomputes exact serialized restart identities across
several scans, and rejects sourced restart records whenever body length changes
(`crates/rdocx-layout/src/engine.rs:1492`,
`crates/rdocx-layout/src/engine.rs:1847`, and
`crates/rdocx-layout/src/engine.rs:1834`). The backlog maps those mechanisms to
F-X084 through F-X086 (`docs/hld/14-development-backlog.md:4481`).

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Performance", specifically retained body
  cache identity, transactional publication, complete-boundary restart
  checkpoints, exact suffix matching, and source provenance.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", specifically the
  editor-scale paragraph-cache and restart-pagination regression gates.
- `docs/hld/14-development-backlog.md`, "F-X083, Close confirmed Issue 67 and
  intake Issue 69".
- GitHub Issue 67, the reporter's v0.12.0 confirmation and explicit closure
  request.
- GitHub Issue 69, the reporter's corrected cause and the four exact offered
  commits.

## Approach

Treat F-X083 as a record and intake story. Make no layout, parser, public API,
dependency, feature, crate, module, or test-fixture change.

Authenticate the live Issue 67 state, reporter identity, confirmation comments,
F-X075 implementation commit `b497d8a6ce90e9c94c06c2a3fc0782e942e75e8d`,
and v0.12.0 release tag `19adaacfcf82e3918bba4f8c3648747f1969b746`.
Close Issue 67 as completed with one maintainer comment linking the reporter's
confirmation, the F-X075 commit, the v0.12.0 release, and Issue 69 as separate
remaining scope. Do not rewrite the historical release record or post another
release-bound notification.

Authenticate the Issue 69 reporter and exact fork commit metadata, then retain
this one-to-one mapping:

| Story | Mechanism | Offered input |
|---|---|---|
| F-X084 | Note-part changes disable paragraph-cache reads document-wide | `4777a74167495a5116289e1f905dfd9ad4dbe807` |
| F-X085 | Restart scans recompute exact body identity | `eff0ea0c28b5eaf08180b09b58e0c0f486b7433b` |
| F-X086 | Enable safe prefix restart after a body-length change, then forbid stale sourced tail reuse | `9e48bc86876c294b8daa314e577e84b6fcd7ac97` and `c8315b92857c951146fc866cd044b214194a09a8` |

Compare only the named patches with current main. Do not merge the external
branch or treat surrounding commits as trusted input. Preserve contributor
credit to `@emptinessform`, but do not copy external co-author trailers into
repository commits. Amend the F-X086 backlog entry to name both enabling and
safety commits. Leave Issue 69 open and otherwise unmodified.

## Rejected alternatives

- Reimplement Issue 67. The reporter confirmed the shipped behavior.
- Close Issue 67 without an evidence-linked comment. The acceptance gate
  requires the closure to cite the confirmation.
- Trust the external branch as a unit. It contains unrelated editor changes.
- Cite only `c8315b92` for F-X086. That patch assumes the enabling
  `9e48bc86` parent and does not remove current main's whole-record veto.
- Adopt an offered patch during F-X083. F-X084 through F-X086 own independent
  implementation and review.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `page_spanning_prose_publishes_complete_boundary_restart_records` | The shipped Issue 67 fix retains the one-pass complete-boundary behavior. |
| regression | `page_spanning_prose_restarts_warm_edits_exactly` | Sourced warm edits retain bounded work and exact fresh-layout equality. |
| external records | authenticated Issue 67 state check | The issue is closed as completed and its comment links every required evidence item. |
| intake | authenticated Issue 69 and fork metadata check | Issue 69 remains open, exact full SHAs are recorded, and authorship is not overstated. |
| regression contract | one-to-one story mapping audit | Each mechanism maps to exactly one pending story, with F-X086 naming both required patches. |

The **test gate is regression**. The existing page-spanning restart regression
passes, the Issue 67 closure cites the reporter's confirmation, and each live
Issue 69 mechanism maps to exactly one pending story with authenticated credit.

## HLD impact

- `docs/hld/14-development-backlog.md`

## Risk routing

none. This story changes a backlog intake record and GitHub Issue 67 state. It
does not change layout behavior, a parser or serializer, crate dependencies,
public API, bindings, assets, features, versions, or release machinery.

## Hash harness

Expected unchanged across all 49 entries. The story changes tracked planning
prose and one GitHub issue state only. Any output delta is unrelated and blocks
completion.

## Implementation checklist

- [x] Reconfirm F-X075 and F-X076 are done.
- [x] Run the deterministic Issue 67 page-spanning regressions.
- [x] Re-read live Issues 67 and 69 immediately before mutation.
- [x] Authenticate the reporter-owned offer comments and exact fork metadata.
- [x] Amend F-X086 to name both `9e48bc86` and `c8315b92`.
- [x] Record the exact F-X084 through F-X086 mechanism mapping.
- [x] Close Issue 67 as completed with one evidence-linked comment.
- [x] Verify Issue 67 is closed and Issue 69 remains open and unmodified.
- [x] Preserve contributor credit without importing co-author trailers.
- [x] Run focused regression, prose, full verification, and hash checks.
- [x] Update exactly the listed HLD file.

## Open questions

None. The user approved correcting F-X086 to the complete `9e48bc86` plus
`c8315b92` mechanism. F-X083 will make exactly one external mutation, closing
Issue 67 with an evidence-linked comment. Issue 69 will receive no mutation.
