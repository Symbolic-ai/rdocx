# S58 sprint review, pass 22

**Reviewed**: `sprint/s58` at
`675f349b0eacf797dfa876ab75ddda2861621245` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 208 files, 24,635 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This twenty-second pass is the explicitly authorized review of the integrated
F-X069 stable recovery preparation. It audits a new carrier, release-note,
registry-proof, and publication-workflow delta after the completed shared 0.8.0
release. It does not repeat pass 21 over an unchanged state.

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
explicitly unclaimed at this dependency-prefix checkpoint. F-X069 is reviewed
but still awaits its separately approved release gate, while F-X070 and F-X031
remain pending at `docs/sprints/CURRENT_SPRINT.md:48` through
`docs/sprints/CURRENT_SPRINT.md:50`. The sprint definition still requires the
complete stable 0.11.1 publication, separately approved v0.11.0 yanks, and the
final branch-protection operation at `docs/sprints/CURRENT_SPRINT.md:83`
through `docs/sprints/CURRENT_SPRINT.md:91`.

The preparation boundary itself holds. Full verification at
`675f349b0eacf797dfa876ab75ddda2861621245` is recorded as passed with 49 of 49
hashes unchanged at `.claude/scratch/S58-run.json:613` through
`.claude/scratch/S58-run.json:617`. The observed gate passed the complete
workspace with the pinned corpora and external oracles, no-default layout,
both WASM targets, warning-free docs, 27 README inventories, all 22 clean
package dry runs, the 10 MiB archive ceiling, and every cargo-deny policy
group. This evidence supports asking for release approval. It does not claim
publication or final sprint completion.

## Not found

- **Shared and stable family interaction, 0 findings**: stable 0.11.1 pins the
  complete published shared 0.8.0 family at `Cargo.toml:34` and
  `Cargo.toml:55` through `Cargo.toml:78`. The packaged stable registry proof
  verifies `rdocx-layout@0.11.1` against registry `oxml-layout@0.8.0` without
  a shared path patch at `scripts/test_sprint_workflow.py:4705` through
  `scripts/test_sprint_workflow.py:4771`. The independent historical proof
  keeps published `rdocx-layout@0.10.1` on `oxml-layout@0.6.0` at
  `scripts/test_sprint_workflow.py:4773`.
- **Publication selection and order, 0 findings**: the stable workflow selects
  exactly `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
  `rdocx-pdf`, `rdocx`, and `rdocx-cli` in dependency order at
  `.github/workflows/publish.yml:61` through
  `.github/workflows/publish.yml:76`. It keeps the 15-package incubating
  allowlist isolated at `.github/workflows/publish.yml:78`, and the carrier
  regression independently derives the exact stable set at
  `scripts/test_sprint_workflow.py:4557` through
  `scripts/test_sprint_workflow.py:4565`.
- **Release notes and contribution inventory, 0 findings**: the reviewed notes
  describe the two-package immutable v0.11.0 attempt, complete 0.11.1 recovery,
  intentional source impacts, and later separately approved yank boundary at
  `CHANGELOG.md:7` through `CHANGELOG.md:90`. They credit Issues 53 and 54 and
  PRs 55 through 58 as hardened equivalents and require all six records to
  remain open at `CHANGELOG.md:92` through `CHANGELOG.md:120`. The regression
  pins both record groups, authenticated handles, four source SHAs, exact
  partial-package set, and leave-open language at
  `scripts/test_sprint_workflow.py:4479` through
  `scripts/test_sprint_workflow.py:4528`.
- **Binding and publication authority, 0 findings**: the Python and rdocx WASM
  metadata track 0.11.1 without entering the seven-package crates.io set. The
  shared packages and `rpptx-wasm` remain at 0.8.0, and all binding, WASM, npm,
  and PyPI publication authority remains excluded at
  `docs/hld/10-bindings-spec.md:726` through
  `docs/hld/10-bindings-spec.md:743`.
- **Partial-release history, 0 findings**: the HLD consistently distinguishes
  the complete shared 0.8.0 publication, last complete stable 0.10.1 family,
  immutable partial 0.11.0 attempt, prepared 0.11.1 recovery, and separately
  approved cleanup at `docs/hld/03-architecture.md:543` through
  `docs/hld/03-architecture.md:565` and
  `docs/hld/14-development-backlog.md:3329` through
  `docs/hld/14-development-backlog.md:3365`.
- **Premature external mutation, 0 findings**: the F-X069 checklist leaves the
  release and external verification steps open at
  `.claude/plans/F-X069-design.md:115` through
  `.claude/plans/F-X069-design.md:124`. The local and remote `v0.11.1` tag is
  absent, and workspace metadata still disables publication, tagging, and
  pushing at `Cargo.toml:44` through `Cargo.toml:51`.
- **Interaction, duplication, layering, dependencies, surface, harness, docs,
  and structure, 0 findings**: F-X069 changes release carriers, reviewed notes,
  HLD current intent, workflow gates, and metadata assertions. It adds no
  runtime parser, serializer, dependency edge, crate, module, feature flag,
  public API, binding method, duplicate release path, or product error path.
  Its feature microscope independently reports 0 defects, 0 smells, and 0
  nitpicks at `.claude/reviews/F-X069-working-pass-3.md:1` through
  `.claude/reviews/F-X069-working-pass-3.md:18`.
