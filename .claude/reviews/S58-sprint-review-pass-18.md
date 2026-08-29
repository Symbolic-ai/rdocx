# S58 sprint review, pass 18

**Reviewed**: `sprint/s58` at
`fc39fb09f0a3c8331cdccb266cf0fd2e8cd2c080` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 199 files, 23,233 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This eighteenth pass reviews the integrated F-X060 stable release preparation.
The release exception deliberately leaves F-X060 reviewed and in progress until
the separately approved `/release v0.11.0` publishes and verifies the selected
family. F-X031 remains ordered after that release at
`.claude/plans/F-X060-design.md:74` through
`.claude/plans/F-X060-design.md:77`.

## Blocking

None. 0 blocking findings.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this scheduled dependency-prefix checkpoint. F-X060 is
still in progress and F-X031 is pending at
`docs/sprints/CURRENT_SPRINT.md:46` through
`docs/sprints/CURRENT_SPRINT.md:47`. Stable 0.11.0 publication remains open at
`docs/sprints/CURRENT_SPRINT.md:75` through
`docs/sprints/CURRENT_SPRINT.md:76`, and the protected `ci-gate` work remains
open at `docs/sprints/CURRENT_SPRINT.md:92` through
`docs/sprints/CURRENT_SPRINT.md:96`.

The dependency prefix has exact-head evidence. The full verification record at
`fc39fb09f0a3c8331cdccb266cf0fd2e8cd2c080` reports 49 of 49 unchanged and
green stable 0.11.0 preparation plus published shared 0.7.0 registry proofs at
`.claude/scratch/S58-run.json:517` through
`.claude/scratch/S58-run.json:520`. This supports the current checkpoint but
does not claim publication, F-X031, or the final M20 gate.

## Not found

- **Stable carriers and family isolation, 0 findings**: the workspace carrier
  is 0.11.0 and the inherited stable pins are 0.11.0, while the shared and
  PowerPoint family remains pinned to 0.7.0 at `Cargo.toml:34` and
  `Cargo.toml:55` through `Cargo.toml:78`. Publication, tagging, and pushing
  remain disabled in preparation metadata at `Cargo.toml:44` through
  `Cargo.toml:51`. The carrier regression enumerates all eleven inherited
  stable carriers and exactly seven publishable stable crates at
  `scripts/test_sprint_workflow.py:4520` through
  `scripts/test_sprint_workflow.py:4554`.
- **Publication order and release authority, 0 findings**: the workflow keeps
  the exact stable dependency order `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`,
  `rdocx-html`, `rdocx-pdf`, `rdocx`, and `rdocx-cli` at
  `.github/workflows/publish.yml:55` through
  `.github/workflows/publish.yml:70`. Hash, metadata, notes, and archive
  preflights precede publication at `.github/workflows/publish.yml:20` through
  `.github/workflows/publish.yml:55`, and mutation tests enforce their order and
  failure propagation at `scripts/test_sprint_workflow.py:6463` through
  `scripts/test_sprint_workflow.py:6514`.
- **Release notes and contribution inventory, 0 findings**: the selected notes
  identify Issues 53 and 54 plus PRs 55 through 58, credit both authenticated
  contributors, record the four reviewed PR source SHAs, classify all six
  outcomes as hardened equivalents, and retain the leave-open policy at
  `CHANGELOG.md:80` through `CHANGELOG.md:108`. The truth-contract regression
  requires all six links, both handles, all four SHAs, exact outcome language,
  and the open-state claim at `scripts/test_sprint_workflow.py:4479` through
  `scripts/test_sprint_workflow.py:4517`.
- **Package and registry proofs, 0 findings**: the current gate inspects the
  normalized `rdocx-layout@0.11.0` archive, requires an unpatched registry
  dependency on exact `oxml-layout@0.7.0`, and resolves that published shared
  version with a fresh Cargo home at `scripts/test_sprint_workflow.py:4694`
  through `scripts/test_sprint_workflow.py:4771`. The independent immutable
  consumer still resolves `rdocx-layout@0.10.1` to `oxml-layout@0.6.0` and
  rejects 0.7.0 at `scripts/test_sprint_workflow.py:4773` through
  `scripts/test_sprint_workflow.py:4828`.
- **No external mutation and release exception, 0 findings**: read-only review
  checks found Issues 53 and 54 and PRs 55 through 58 still open, with no local
  or remote `v0.11.0` tag and no GitHub `v0.11.0` release. That is the required
  pre-release state. `/release` owns the separate immediate approval at
  `.claude/commands/release.md:87` through `.claude/commands/release.md:97`, then
  publication and leave-open notifications at `.claude/commands/release.md:99`
  through `.claude/commands/release.md:124`. F-X060 therefore correctly remains
  reviewed in sprint state at `.claude/scratch/S58-run.json:88` through
  `.claude/scratch/S58-run.json:99` and in progress in the delivery trackers at
  `docs/sprints/BACKLOG.md:515` and `docs/sprints/CURRENT_SPRINT.md:46`.
- **HLD and release-record consistency, 0 findings**: exactly the five
  plan-listed HLD files changed, matching `.claude/plans/F-X060-design.md:104`
  through `.claude/plans/F-X060-design.md:110`. The current testing contract
  records the prepared carriers, current 0.7.0 registry proof, and independent
  historical 0.6.0 proof at `docs/hld/12-testing-strategy.md:1159` through
  `docs/hld/12-testing-strategy.md:1176`. The build contract distinguishes the
  prepared stable 0.11.0 carriers from the last published stable 0.10.1 family
  and leaves external actions to `/release` at
  `docs/hld/15-build-and-toolchain.md:371` through
  `docs/hld/15-build-and-toolchain.md:401`.
- **Interactions, duplication, layering, dependencies, surface, and structure,
  0 findings**: F-X060 changes release carriers, assertions, notes, and the five
  approved HLD files without changing runtime behavior or adding a dependency,
  feature flag, module, test binary, or public API. The selected compatibility
  notes accurately retain prior S58 source-impact and non-rendering boundaries
  at `CHANGELOG.md:64` through `CHANGELOG.md:78`. The feature-level microscope
  is clean at `.claude/reviews/F-X060-working-pass-1.md:3` through
  `.claude/reviews/F-X060-working-pass-1.md:18`.
