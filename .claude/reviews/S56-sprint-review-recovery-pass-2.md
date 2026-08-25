# S56 sprint review, recovery pass 2

**Reviewed**: `sprint/s56` at
`97e3076df8addc6a82a0e1bb4fbaab5493fc45ae` against merge base
`92659e7ba3742aab888a8d5603e42560ff3398fc`, 150 files, 28,047 additions,
2,707 deletions, and 30,754 changed lines across 26 crates
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Dispositions**: 0 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Bound extension

This review continues the recovery sequence under the user's explicit
authorization to use as much budget as needed to land S56. That authorization
is the recorded decision to extend the default three-pass bound required by
the sprint-review refusal rule (`.claude/commands/sprint-review.md:86`). This
file does not overwrite any earlier review record.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Recovery pass 1 remediation

S1 is fully resolved. The live sprint wave identifies F-X055 as the archived
immutable partial v0.10.0 attempt and F-X057 as the active recovery owner
(`docs/sprints/CURRENT_SPRINT.md:44`). Its sequencing now assigns the complete
v0.10.1 stable publication, all nine contribution notifications, and the six
authorized pull-request closures to F-X057
(`docs/sprints/CURRENT_SPRINT.md:56`). The roadmap records the same ownership
and preserves F-X055 only as the two-package partial attempt
(`docs/sprints/SPRINT_PLAN.md:1018`). The HLD already assigns that exact
recovery outcome to F-X057 (`docs/hld/14-development-backlog.md:3236`).

The post-review delta contains only the recovery pass-1 record and those two
sprint-document corrections. It changes no source, manifest, lockfile,
workflow, release note, HLD, dependency, binding, package asset, or delivery
ledger. The correction therefore reconciles current sequencing without
rewriting the immutable AS_BUILT history.

The earlier failure-atomic consolidation also remains intact. RTF, EPUB, ODT,
and encrypted saves still use the single crate-private atomic writer, which
uses create-new sibling staging, a complete write and file sync, portable
replacement, and failed-stage cleanup (`crates/rdocx/src/document.rs:5725`).
The RTF, EPUB, ODT, and encrypted-save regressions all passed at this SHA and
prove that a failed staging or serialization path preserves the existing
destination (`crates/rdocx/src/lib.rs:221`, `crates/rdocx/src/epub.rs:4106`,
`crates/rdocx/src/odt.rs:7308`, `crates/rdocx/src/document.rs:6491`).

## Exact release readiness

The prepared release content remains coherent at the reviewed SHA. Cargo
metadata reports exactly seven publishable stable packages at 0.10.1 and 15
publishable incubating packages at 0.6.0. The workspace version and internal
pins state the same split (`Cargo.toml:33`, `Cargo.toml:55`). No `oxml-*`
package has a reverse dependency on an `rdocx-*` or `rpptx-*` package. The
stable workflow publishes the exact seven-package allowlist in dependency
order after its metadata, registry-edge, notes, and archive preflights
(`.github/workflows/publish.yml:23`, `.github/workflows/publish.yml:55`).

The clean registry-edge regression passed again. It packages
`rdocx-layout@0.10.1` against crates.io while leaving `oxml-layout@0.6.0`
unpatched, which reproduces and closes the exact failure that stopped v0.10.0
(`scripts/test_sprint_workflow.py:4247`). The stable version contract and all
five focused release-note truth tests passed. Those tests enforce the exact
v0.10.0 partial inventory, direct versus hardened-equivalent classification,
record-to-contributor binding, and complete stable inventory, including
rejection of reversed landing truth, expanded partial inventory, and swapped
credits (`scripts/test_sprint_workflow.py:4073`,
`scripts/test_sprint_workflow.py:4285`,
`scripts/test_sprint_workflow.py:4299`,
`scripts/test_sprint_workflow.py:4312`,
`scripts/test_sprint_workflow.py:4325`). The reviewed notes continue to state
that v0.10.0 contains only `rdocx-opc` and `rdocx-oxml`, while v0.10.1 is the
first complete stable S56 family (`CHANGELOG.md:16`, `CHANGELOG.md:69`).

The exact `v0.10.1` tag is absent locally and from `origin`. The focused
release-note check, metadata audit, unpatched registry proof, prose check,
generated-skill drift check, and diff check all pass. The deterministic hash
harness also passes with all 49 entries unchanged. The last recorded full
verification covers the unchanged release implementation at
`7f380652491283cf78610628c3888c280e725b42`
(`.claude/scratch/S56-run.json:138`). Because the review record itself must land
before release preflight, `/release` must still require its normal full-gate
and clean-review records at the exact SHA selected for the tag.

## Human action

### H1, v0.10.1 remains behind separate final approval

`.claude/commands/release.md:87`
`.claude/plans/F-X057-design.md:132`

This review authorizes no release mutation. After this review is landed and
the release preconditions cover the exact SHA to be tagged, `/release v0.10.1`
must present the SHA, seven-package set, rendered notes, nine-record inventory,
and exact comments, then obtain a separate explicit go or no-go immediately
before the first push or tag mutation. Earlier sprint authorization does not
satisfy that boundary.

**Disposition**: human-action at the final release approval boundary.

## Milestone gate

The M18 gate is: "each format round-trips at its declared fidelity level, and
every lossy conversion records a diagnostic naming what it dropped"
(`docs/hld/14-development-backlog.md:1457`).

The technical gate remains satisfied. ODT output reopens through the F-179
reader and compares the supported document structure
(`crates/rdocx/tests/integration_test.rs:546`), while the loss-matrix regression
retains supported siblings and checks exact diagnostics
(`crates/rdocx/src/odt.rs:6769`). EPUB spine and navigation retain source
outline order (`crates/rdocx/src/epub.rs:3987`), and the checksum-pinned
EPUBCheck 5.3.0 oracle validates the source-built publication
(`crates/rdocx/src/epub.rs:5543`). The SVG golden compares the representative
page with the PNG backend at 150 dpi and proves its 0.99 SSIM threshold rejects
a one-point perturbation (`crates/rdocx/src/svg.rs:2208`). The post-review delta
changes none of these implementations or gates, and the current hash harness
remains byte-identical across all 49 entries.

## Not found

- `interaction`: F-X057 consumes the verified shared 0.6.0 family through the
  exact unpatched registry edge, and the sequencing documents now agree on the
  release owner.
- `duplication`: no runtime helper, release allowlist, release-note inventory,
  or delivery record is duplicated by the post-review delta.
- `layering`: Cargo metadata contains no prohibited reverse family edge.
- `harness`: no baseline changed, and all 49 entries match.
- `gate`: the M18 technical evidence remains intact, and the focused release
  and failure-atomic checks pass at the reviewed SHA.
- `docs`: CURRENT_SPRINT, SPRINT_PLAN, the HLD, design plan, changelog, and
  release workflow now describe one consistent recovery sequence.
- `deps`: no dependency changed after recovery pass 1. Stable requirements
  remain 0.10.1 or the published shared 0.6.0 family.
- `surface`: the post-review delta adds no Rust, Python, WASM, CLI, parser,
  serializer, crate, module, feature, or public API surface.
- `release-note binding`: the exact recovery facts, record sets, contributor
  attribution, and hardened-equivalent classification remain positively
  enforced and mutation-tested.
