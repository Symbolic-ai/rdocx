# S51 sprint review, pass 3

**Reviewed**: `sprint/s51` at `f30f34f0e6baacd99baaaddef945e85aaf962974`
against merge base `cd3b34109e8d45da7d06a11d11964971c8d1568d`,
150 files and 18,739 changed lines. Crates: `oxml-chart`,
`oxml-cli-support`, `oxml-core`, `oxml-drawing`, `oxml-layout`,
`oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx-layout`,
`rdocx-oxml`, `rdocx-wasm`, `rdocx`, `rpptx-chart`, `rpptx-cli`,
`rpptx-layout`, `rpptx-oxml`, `rpptx-render`, `rpptx-wasm`, and `rpptx`

**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the stable release notes omit Issue 37 attribution

`CHANGELOG.md:156`

The stable notes describe the complete `WordLayoutResult` surface at
`CHANGELOG.md:37`, but the Contributors section thanks `@emptinessform` only
for the Issue 39 relayout measurements and cache proposal. GitHub Issue 37 was
authored by the same verified reporter, directly requested the complete layout
result that F-X032 delivered, and remains open awaiting the stable 0.8.0
release. The integrated delivery record identifies F-X032 as the Issue 37
contract at `docs/sprints/AS_BUILT.md:7812`, while the F-X036 plan requires the
selected-range notes to credit verified issue reporters at
`.claude/plans/F-X036-design.md:54`.

Crediting the reporter for a different issue does not attribute the
complete-layout contribution that this release is publishing. This violates
the reviewed contributor-credit contract and the sprint definition that every
release carry contributor credit at `docs/sprints/CURRENT_SPRINT.md:84`. Update
the stable Contributors section to credit `@emptinessform` for Issue 37 and its
complete-layout report, and add mutation-sensitive coverage so removing that
attribution fails. Issue 38 is already explicitly credited in the published
`rpptx-v0.4.0` notes, so this finding is limited to the previously uncredited
Issue 37 contribution.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M16 gate is: "a template with loops, conditionals and a repeating table row
produces a correct document from a JSON data model, and every field in it
evaluates to the value Word computes" at
`docs/hld/14-development-backlog.md:1299`.

The product gate holds at the reviewed SHA. The nested loop and conditional
fixture at `crates/rdocx/tests/regression_test.rs:3323`, the repeating table and
record fixtures beginning at `crates/rdocx/tests/regression_test.rs:3389`, and
the pinned field corpus all pass through the recorded full workspace gate. The
same exact-SHA record covers mail merge, comparison, watermarks, complete layout,
provenance, warm relayout, and ordered body access.

S51 is not ready for the stable release because B1 leaves the release-note
definition incomplete. This is the default third and final sprint-review pass.
After remediation changes the SHA, a new full verification and an explicit
decision to extend the review bound are required before any release approval.

## Evidence

- Sprint state records F-X036 as reviewed at
  `.claude/scratch/S51-run.json:91` and a passing `/verify --full` with an
  unchanged 49-of-49 harness at this exact HEAD at
  `.claude/scratch/S51-run.json:171`. The record therefore supports the
  release gate it claims, but it cannot override B1.
- Independent focused checks at this SHA report all 49 hash entries unchanged,
  all 65 workflow regressions passing, release-note validation passing, prose
  and generated-skill gates passing, and no diff-check error.
- Metadata reports the exact seven publishable stable crates at 0.8.0 and the
  four unpublished workspace-version carriers. All nine stable internal pins
  are 0.8.0, all explicit incubating packages remain 0.4.0, and no `oxml-*`
  package has a forbidden `rdocx-*` or `rpptx-*` dependency. The stable
  allowlist remains exact and dependency ordered at
  `.github/workflows/publish.yml:55`.
- All 15 incubating dependencies resolve from crates.io at 0.4.0. The annotated
  `rpptx-v0.4.0` tag targets the reviewed SHA recorded at
  `docs/sprints/AS_BUILT.md:7956`, and the published family remains separate
  from the prepared stable carriers.
- The exact `v0.8.0` tag is absent locally and from `origin`, all seven selected
  0.8.0 versions are absent from crates.io, and no stable GitHub release exists.
  The release boundary therefore remains unmutated.
- PR 36 remains merged into `sprint/s51` through merge commit
  `92951e71474383b48ce7ede194be4d0f34729488`, whose second parent is Pedro
  Assumpcao's original commit
  `79390535acba0a116b25ac986b863bdb941c8f15`. The public delivery and current
  base CI evidence remain recorded at `docs/sprints/AS_BUILT.md:7913` and
  `docs/sprints/AS_BUILT.md:7933`.
- Issue 37's F-X032 surface remains source-bundled, caller-font aware, and
  cache-isolated at `docs/sprints/AS_BUILT.md:7793`. Issue 38 provenance remains
  exact and result local at `docs/sprints/AS_BUILT.md:7756`. Issue 39 reuse
  remains bounded, transactional, diagnostic preserving, and provenance safe
  at `docs/sprints/AS_BUILT.md:7869`. The reporter's later engine-transfer
  proposal is outside the approved F-X038 public surface, and the release notes
  make no claim that it shipped.

## Not found

- `interaction`: the pass-1 watermark and cache interaction remains fixed.
  Mail merge, comparison, watermarks, complete-layout ownership, provenance,
  relayout caching, and ordered body access retain compatible mutation and
  cache boundaries.
- `duplication`: no duplicate sprint subsystem or competing release-note source
  was added.
- `layering`: no forbidden crate edge was added.
- `harness`: no baseline differs from the merge base, and all completed feature
  records agree with the independent 49-entry result.
- `gate`: the M16 product gate has direct passing evidence. B1 is a release
  readiness failure, not a product-gate failure.
- `docs`: outside B1, the plan union, HLD, sprint contract, and delivery records
  agree on public surfaces, cache ownership, preservation, package versions,
  and prepared versus published release state.
- `deps`: no external dependency was added. The already published incubating
  0.4.0 graph satisfies the prepared stable graph.
- `surface`: every integrated public type, field, and method belongs to an
  approved S51 story. Python, WASM, CLI, npm, and PyPI exposure remains outside
  the stable crates.io release.
- `structure`: no unowned trait, generic, feature flag, crate, module, file, or
  forwarding wrapper was introduced after the clean second sprint review.
