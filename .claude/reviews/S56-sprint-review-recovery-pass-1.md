# S56 sprint review, recovery pass 1

**Reviewed**: `sprint/s56` at
`7f380652491283cf78610628c3888c280e725b42` against merge base
`92659e7ba3742aab888a8d5603e42560ff3398fc`, 149 files, 27,890 additions,
2,707 deletions, and 30,597 changed lines across 26 crates
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have
**Dispositions**: 1 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Bound extension

The user explicitly authorized as much budget as needed to land S56. That is
the recorded decision to extend the default three-pass bound for this recovery
review, as required by the sprint-review refusal rule
(`.claude/commands/sprint-review.md:86`). This file starts a separately named
recovery sequence and does not overwrite passes 1 through 3.

## Blocking

None.

## Should-fix

### S1, current sprint sequencing still assigns the release to archived F-X055

`docs/sprints/CURRENT_SPRINT.md:56`
`docs/sprints/SPRINT_PLAN.md:1018`

The live wave marks F-X055 archived, F-X056 done, and F-X057 in progress, and
the recovery paragraph correctly assigns the complete stable 0.10.1 release to
F-X057. Two earlier sequencing paragraphs still say that F-X055 runs last,
publishes the exact stable family at v0.10.0, and owns the post-publication pull
request closure. That outcome is impossible because the v0.10.0 tag and two
published package versions are immutable and F-X055 is archived after its
partial result. A future session following the stale prose could target the
non-reusable tag or the wrong release F-ID for contribution closure.

Rewrite the current sprint sequencing prose to state that F-X055 ended as the
immutable partial attempt and that F-X057 owns the complete stable publication,
nine notifications, and six authorized pull-request closures at v0.10.1. Keep
the append-only AS_BUILT history unchanged.

**Disposition**: fix-now before final release approval.

## Nice-to-have

None.

## Release recovery audit

The external and repository release truth agree. The remote annotated
`v0.10.0` tag dereferences to
`aa44a65629a5ce2c56852582af2ea89e11069b52`. crates.io contains
`rdocx-opc@0.10.0` and `rdocx-oxml@0.10.0`, does not contain
`rdocx-layout@0.10.0`, and GitHub has no v0.10.0 release. The current HLD
records that exact immutable partial outcome and hands recovery to F-X056 and
F-X057 (`docs/hld/14-development-backlog.md:3182`).

The incubating prerequisite is complete. The remote annotated
`rpptx-v0.6.0` tag dereferences to
`55fb2f54caf91d7dedc8936b4c7b116354590628`, its GitHub release is live, and
the completion record binds all 15 registry packages, owners, release-body
bytes, and three notifications to that reviewed SHA
(`docs/sprints/AS_BUILT.md:9178`).

The stable recovery is prepared coherently at the reviewed SHA. Workspace and
stable pins are 0.10.1 while every shared and Presentation pin remains 0.6.0
(`Cargo.toml:33`, `Cargo.toml:53`). Cargo metadata reports exactly the seven
publishable stable packages at 0.10.1 and the 15 publishable incubating packages
at 0.6.0, with no reverse `oxml-*` family edge. The clean registry regression
packages `rdocx-layout@0.10.1`, leaves `oxml-layout@0.6.0` unpatched, and passed
against crates.io (`scripts/test_sprint_workflow.py:4247`). The stable workflow
allowlist publishes exactly seven packages in dependency order after its
metadata, registry-edge, notes, and archive preflights
(`.github/workflows/publish.yml:23`, `.github/workflows/publish.yml:55`).

The v0.10.1 notes passed both deterministic check and render modes. They state
the two-package v0.10.0 partial inventory exactly, identify 0.10.1 as the first
complete stable S56 family, and preserve the pre-1.0 compatibility guidance
(`CHANGELOG.md:16`, `CHANGELOG.md:69`). Each of the nine selected GitHub records
appears twice. Live record authors bind Issue 44, PR 45, and Issue 46 to
`@emptinessform`, and PRs 47 through 52 to `@pedroassumpcao`. The exact truth
contract binds those full record sets to the matching contributor paragraphs
and the all-hardened-equivalent classification
(`scripts/test_sprint_workflow.py:3989`). Reversed landing truth, an expanded
partial package inventory, and swapped contributor credits are all rejected by
dedicated mutations (`scripts/test_sprint_workflow.py:4299`). The nine prepared
comments name v0.10.1, the exact hardened outcome, and the authenticated
contributor. They remain unposted until publication verifies.

The exact requested tag `v0.10.1` is absent locally and from `origin`. Full
verification is recorded as passing at the reviewed SHA with all 49 hashes
unchanged (`.claude/scratch/S56-run.json:138`). This recovery changes release
metadata, dependency requirements, tests, and current-reality documentation.
It adds no runtime API or format behavior.

## Human action

### H1, v0.10.1 remains behind separate final approval

`.claude/commands/release.md:87`
`.claude/plans/F-X057-design.md:132`

No release mutation belongs in this review. After S1 is fixed and a clean
current-HEAD recovery review is recorded, `/release v0.10.1` must present the
exact final SHA, seven-package set, rendered notes, nine-record inventory, and
nine exact comments, then obtain a new explicit go or no-go immediately before
the first push or tag mutation. Earlier sprint authorization does not satisfy
that boundary.

**Disposition**: human-action after a clean recovery pass at the final SHA.

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
a one-point perturbation (`crates/rdocx/src/svg.rs:2208`). F-X056 and F-X057
change no format implementation, and the exact-SHA full gate and 49-entry hash
harness pass.

## Not found

- `interaction`: the published shared 0.6.0 family supplies the exact layout
  API the stable 0.10.1 graph consumes, and the unpatched registry-edge proof
  closes the failure hidden by the local workspace graph.
- `duplication`: no new runtime implementation, release family, allowlist, or
  contribution inventory is duplicated by F-X057.
- `layering`: Cargo metadata contains no reverse `oxml-*` dependency on an
  `rdocx-*` or `rpptx-*` crate.
- `harness`: no baseline file changed, and the current-HEAD harness check passes
  all 49 entries unchanged.
- `gate`: the M18 technical tests, full verification, exact package dry run,
  release-note checks, registry dependency proof, bindings, assets, WASM, and
  supply-chain riders are recorded clean at the reviewed SHA.
- `deps`: every stable internal requirement is 0.10.1 or the published shared
  0.6.0 family. No external runtime dependency was added.
- `surface`: F-X057 adds no public Rust, Python, WASM, CLI, parser, serializer,
  crate, module, feature, or runtime behavior.
- `release-note binding`: exact recovery facts, record counts, contributor to
  record attribution, and hardened-equivalent classification are positively
  enforced and mutation-tested.
