# S39 sprint review, pass 3

**Reviewed**: `sprint/s39` at
`1b13dbe4a5454f1d1629ff8915287b26daa10ed0` against merge base
`302ce2a4ece215227d1b1bf0338e266a58a41dbd`, 81 files and 2,770 changed
lines, crates: all 26 workspace packages
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Dispositions**: 0 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Earlier dispositions

- Pass 1 was clean for the stable preparation boundary and required later
  reviews after each release-state change
  (`.claude/reviews/S39-sprint-review-pass-1.md:3`,
  `.claude/reviews/S39-sprint-review-pass-1.md:115`). F-X010 is now completed
  with the exact seven stable crates at 0.6.0, the reviewed tag peel, successful
  workflow jobs, registry ownership, and rendered READMEs
  (`docs/sprints/AS_BUILT.md:5657`,
  `docs/sprints/AS_BUILT.md:5663`,
  `docs/sprints/AS_BUILT.md:5669`,
  `docs/sprints/AS_BUILT.md:5685`).
- Pass 2 was clean for the incubating implementation SHA and required a fresh
  exact-SHA verification after its tracked review record changed HEAD
  (`.claude/reviews/S39-sprint-review-pass-2.md:3`,
  `.claude/reviews/S39-sprint-review-pass-2.md:83`,
  `.claude/reviews/S39-sprint-review-pass-2.md:143`). The only new tracked delta
  since that implementation review is the pass 2 review file itself. This pass
  reviews that final preparation state after full verification at the resulting
  current HEAD.

## Integrated release preparation

- The three S39 stories remain jointly coherent. F-X009 supplies the exact
  26-package README contract and 21 publishable archive inventories
  (`docs/hld/14-development-backlog.md:1238`,
  `docs/hld/14-development-backlog.md:1246`). F-X010 supplies the completed
  stable 0.6.0 prerequisite without granting authority to bindings, WASM,
  Python, npm, PyPI, or incubating crates
  (`docs/hld/14-development-backlog.md:1251`,
  `docs/hld/14-development-backlog.md:1257`). F-X011 then prepares only the
  incubating family for its separate release
  (`docs/hld/14-development-backlog.md:1270`,
  `docs/hld/14-development-backlog.md:1275`).
- Fresh Cargo metadata reports exactly 26 workspace packages and 21
  publishables. Exactly 15 preparation members are at 0.2.0, of which the exact
  14 crates.io packages are publishable and `rpptx-wasm` is not. Exactly eleven
  stable-train members remain at 0.6.0, of which seven are publishable. The
  regression owns the 14-package list, adds the WASM preparation member,
  verifies all pins and lock entries, fixes the stable workspace at 0.6.0, and
  rejects WASM publication (`scripts/test_sprint_workflow.py:3102`,
  `scripts/test_sprint_workflow.py:3119`,
  `scripts/test_sprint_workflow.py:3122`,
  `scripts/test_sprint_workflow.py:3131`,
  `scripts/test_sprint_workflow.py:3144`).
- The README and archive contracts remain version-sensitive and exact. The
  incubating regression checks every README carrying a 0.2.0 installation or
  dependency literal (`scripts/test_sprint_workflow.py:3154`). The runner derives
  the complete publishable local patch identity set from metadata, rejects any
  missing or extra identity, requires 26 distinct README sources and 21
  publishable packages, and verifies every resulting archive
  (`scripts/readme_doctests.py:273`,
  `scripts/readme_doctests.py:296`,
  `scripts/readme_doctests.py:443`,
  `scripts/readme_doctests.py:450`,
  `scripts/readme_doctests.py:470`). A fresh run compiled the 26 Rust examples
  and validated all 26 sources and 21 inventories.
- The publish workflow preserves the family boundary. It runs the stable 0.6.0
  and incubating 0.2.0 metadata regressions before the exact 21-package patched
  dry run, then chooses only the requested dependency-ordered allowlist
  (`.github/workflows/publish.yml:20`,
  `.github/workflows/publish.yml:23`,
  `.github/workflows/publish.yml:26`,
  `.github/workflows/publish.yml:51`,
  `.github/workflows/publish.yml:68`). Fresh dependency inspection finds no
  prohibited family edge. The only `oxml-*` dependency on an `rdocx-*` or
  `rpptx-*` crate is the documented `oxml-drawing -> rdocx-oxml` Theme adapter
  (`docs/hld/03-architecture.md:41`,
  `docs/hld/03-architecture.md:55`).

## Verification evidence

- The sprint authority records successful `/verify --full` at the exact
  reviewed HEAD with the hash harness unchanged
  (`.claude/scratch/S39-run.json:58`,
  `.claude/scratch/S39-run.json:60`,
  `.claude/scratch/S39-run.json:61`). The newly observed run passed all eleven
  canonical steps: formatting, workspace Clippy, changed-crate tests, workspace
  tests, all 28 hashes, prose and generated-skill synchronization,
  no-default-features, both WASM targets, documentation and README validation,
  the patched package dry run, and supply-chain policy
  (`.claude/commands/verify.md:11`,
  `.claude/commands/verify.md:25`,
  `.claude/commands/verify.md:50`,
  `.claude/commands/verify.md:53`,
  `.claude/commands/verify.md:97`).
- Fresh independent focused checks passed all 38 workflow tests, the README
  runner, both locked WASM target checks, all 28 deterministic hashes,
  formatting, prose, generated-skill synchronization, dependency inspection,
  sprint-delta diff hygiene, and `cargo deny check`. The supply-chain command
  reports only the existing permitted warnings and finishes with advisories,
  bans, licenses, and sources all accepted, matching the story gate
  (`docs/hld/14-development-backlog.md:1281`,
  `docs/hld/14-development-backlog.md:1284`).
- Every one of the 21 current publishable archives is present and below 10 MiB.
  `oxml-layout` contains 20 TTF files and four legal files, `rdocx-layout`
  contains no duplicated font or legal payload, and `rpptx` contains
  `assets/default.pptx`. This matches both the package contract and release
  precondition (`docs/hld/15-build-and-toolchain.md:187`,
  `docs/hld/15-build-and-toolchain.md:188`,
  `.claude/commands/release.md:57`,
  `.claude/commands/release.md:62`).

## HLD, checklist, and delivery state

- F-X011 lists exactly HLD03, HLD14, and HLD15 as its impact
  (`.claude/plans/F-X011-design.md:68`). HLD03 describes the published immutable
  0.1.3 family, prepared unpublished 0.2.0 family, and separately approved
  release boundary (`docs/hld/03-architecture.md:123`,
  `docs/hld/03-architecture.md:127`). HLD14 states the exact 15-member
  preparation, 14-package publication set, unchanged stable family, and pending
  external verification (`docs/hld/14-development-backlog.md:1270`,
  `docs/hld/14-development-backlog.md:1281`,
  `docs/hld/14-development-backlog.md:1286`). HLD15 agrees on the exact
  allowlist, version state, archive contract, and release authority
  (`docs/hld/15-build-and-toolchain.md:159`,
  `docs/hld/15-build-and-toolchain.md:226`,
  `docs/hld/15-build-and-toolchain.md:239`). No unlisted HLD section is
  contradicted.
- The implementation checklist marks preparation and verification complete
  (`.claude/plans/F-X011-design.md:96`,
  `.claude/plans/F-X011-design.md:105`). Its remaining items are correctly the
  current clean review, separate approval, publication observation, and
  post-publication finalization
  (`.claude/plans/F-X011-design.md:107`,
  `.claude/plans/F-X011-design.md:109`,
  `.claude/plans/F-X011-design.md:111`). This pass supplies the clean review at
  the exact release SHA. The other unchecked items remain future actions, not
  stale claims.
- The delivery ledgers keep F-X009 and F-X010 done while F-X011 remains in
  progress under `codex` (`docs/sprints/CURRENT_SPRINT.md:24`,
  `docs/sprints/CURRENT_SPRINT.md:25`,
  `docs/sprints/CURRENT_SPRINT.md:26`,
  `docs/sprints/BACKLOG.md:298`). Only the two completed stories appear in
  `AS_BUILT.md` and `SPRINT_TRACKER.md`
  (`docs/sprints/SPRINT_TRACKER.md:223`,
  `docs/sprints/SPRINT_TRACKER.md:224`). The run remains in review phase and
  records F-X011 as reviewed rather than completed
  (`.claude/scratch/S39-run.json:16`,
  `.claude/scratch/S39-run.json:20`,
  `.claude/scratch/S39-run.json:26`). This is exactly the release command's
  pre-publication state (`.claude/commands/release.md:45`,
  `.claude/commands/release.md:46`).

## Release and PR boundaries

- Live inspection finds `rpptx-v0.2.0` absent locally, absent from `origin`,
  and absent from GitHub Releases. Every exact incubating crates.io 0.2.0 lookup
  is absent. The npm 0.2.0 package and both PyPI 0.2.0 projects are also absent.
  All seven stable crates remain available at 0.6.0, and the annotated `v0.6.0`
  tag still peels to its reviewed SHA. This is the required pre-release state,
  and HLD15 prohibits interpreting preparation as publication authority
  (`docs/hld/15-build-and-toolchain.md:230`,
  `docs/hld/15-build-and-toolchain.md:246`).
- PR 25 remains merged under contributor Jon Stokes, and its merge commit is an
  ancestor of the reviewed S39 HEAD. The contributor credit and merge note
  remain in the durable delivery record
  (`docs/sprints/AS_BUILT.md:5542`,
  `docs/sprints/AS_BUILT.md:5569`). No S39 release preparation rewrote that
  boundary.

## Human action

The one remaining pre-release disposition is the separate final
`/release rpptx-v0.2.0` approval at
`1b13dbe4a5454f1d1629ff8915287b26daa10ed0`. Earlier sprint authorization does
not count. The release command must report the exact SHA, tag, package set,
version, remote, and workflow, then receive an immediate explicit go before its
first external mutation (`.claude/commands/release.md:75`,
`.claude/commands/release.md:77`). This is a required human action, not a defect,
should-fix, or nice-to-have finding.

## Milestone gate

The S39 gate remains deliberately one external release short of completion.
The documentation, example, archive, exact full-verification, unchanged-hash,
stable 0.6.0 publication, and publication-exclusion requirements hold
(`docs/sprints/CURRENT_SPRINT.md:37`,
`docs/sprints/CURRENT_SPRINT.md:40`,
`docs/sprints/CURRENT_SPRINT.md:42`,
`docs/sprints/CURRENT_SPRINT.md:43`,
`docs/sprints/CURRENT_SPRINT.md:44`,
`docs/sprints/CURRENT_SPRINT.md:47`). The fourteen incubating 0.2.0 registry
publications and rendered crates.io READMEs remain pending
(`docs/sprints/CURRENT_SPRINT.md:45`). Their absence is expected because each
family has its own full verification, clean review, and immediate approval
boundary (`docs/sprints/SPRINT_PLAN.md:613`,
`docs/sprints/SPRINT_PLAN.md:615`). This pass is clean for the F-X011 release
approval boundary. It does not claim that S39 is ready to close.

## Not found

No cross-feature interaction defect, duplicate helper, prohibited dependency
edge, undeclared hash delta, false milestone claim, version or lock mismatch,
stable-family regression, incubating set error, README coverage or archive
error, asset omission, supply-chain failure, HLD scope conflict, stale
checklist completion, unrequested public surface, unauthorized tag or
publication, PR credit loss, ledger drift, run-state mismatch, prose violation,
or release-boundary bypass was found.
