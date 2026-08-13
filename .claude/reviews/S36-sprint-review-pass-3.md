# S36 sprint review, pass 3

**Reviewed**: `sprint/s36` at
`d56e106c96b9eec6ba3cecf4e7191d90ac0dc5e4` against merge base
`3c2a019fffccbdd7c7e6465c3c004a74c75dc486`, 67 files, 5,437 insertions
and 966 deletions, crates: `oxml-cli-support`, `rdocx`, `rdocx-cli`,
`rdocx-wasm`, `rpptx`, `rpptx-cli`, and `rpptx-wasm`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Run-sprint disposition

- `fix-now`: none.
- `tracked-follow-up`: none. F-X006 already owns the deferred Rust release.
- `human-action`: none for S36. F-X006 retains the separate final release
  approval before any registry mutation.
- `refuted`: none.

## Earlier finding

### B1, resolved

The complete record now states one consistent publication boundary.
`oxml-cli-support` and `rpptx-cli` are publishable and currently unpublished in
their completed plan approaches at `.claude/plans/F-143-design.md:29` and
`.claude/plans/F-144-design.md:30`. AS_BUILT uses the same wording at
`docs/sprints/AS_BUILT.md:5235` and `docs/sprints/AS_BUILT.md:5269`, while the
S36 summary records that no Rust or npm package was published at
`docs/sprints/SPRINT_TRACKER.md:53`.

The delivery arithmetic and deferred work also reconcile. S36 records eight
planned, eight done, zero carried, thirteen estimated days, and eight actual
days at `docs/sprints/SPRINT_TRACKER.md:53`. Its eight feature rows total the
same estimates and actuals at `docs/sprints/SPRINT_TRACKER.md:210`. The backlog
reports 160 stories, 159 done, and one pending at
`docs/sprints/BACKLOG.md:33`, with F-X006 pending in S37 at
`docs/sprints/BACKLOG.md:293`. The variance record now retains that one
fresh-version release story at `docs/sprints/SPRINT_TRACKER.md:309`.

S36 is explicitly the v1 implementation gate, with registry publication
deferred, at `docs/sprints/SPRINT_PLAN.md:562`. F-X006 owns a fresh common
version above 0.1.2 and the separately approved 14-package release at
`docs/hld/14-development-backlog.md:1176`. The earlier `rpptx-v0.1.2` tag and
its twelve published packages remain immutable at
`docs/hld/14-development-backlog.md:1180`. HLD 03 confirms that the two new
packages remain unpublished at 0.1.2 at `docs/hld/03-architecture.md:136`.

No existing version or tag is assigned new content, and this remediation made
no registry, release, tag, authentication, or publication mutation.

## Sprint definition of done

All nine S36 definition-of-done items hold.

- The shared CLI contract binds `2,4-6`, schema one, range resource limits, and
  output paths in seven tests at `crates/oxml-cli-support/src/lib.rs:129`.
- The presentation executable contains the seven F-144 commands plus F-145
  `thumbnail` and `outline` at `crates/rpptx-cli/src/main.rs:18`. Its fourteen
  integration tests cover corrupted and 50-deck validation, deterministic
  rendering, resource limits, replacement preservation, and both added
  commands.
- The document CLI has one compiled-command integration for each of its seven
  commands, including document-order text and deterministic render branches.
- The WASM CI gate builds both locked release bundler packages, packs and
  installs each tarball into its own fresh consumer, checks inventory, and
  imports the installed module at `.github/workflows/ci.yml:125`. No npm publish
  or registry authority is present in that job.
- The sole README runner requires exactly six `rust,no_run` fences, discovers
  the locked rdocx rlib, and invokes warning-denied rustdoc at
  `scripts/readme_doctests.py:20`.
- `generate_all_samples` is the sole harness generator. The obsolete duplicate
  is deleted, and all 28 hashes remained unchanged.
- The file round-trip test includes process identity in its output path at
  `crates/rdocx/tests/integration_test.rs:145`, and two concurrent invocations
  passed.
- The integrated full verification at the implementation HEAD passed with all
  28 hashes unchanged. Every later commit changed only sprint plans, reviews,
  HLD, and delivery records. Pass 3 also found zero prose violations, all 25
  generated skills in sync, a passing incubating metadata regression, and no
  diff-hygiene error.
- All eight S36 rows are done and unowned at
  `docs/sprints/CURRENT_SPRINT.md:32`. Each plan is completed with every
  checklist item ticked, each latest feature review reports zero defects and
  zero smells, and each story has exactly one tracker row and one AS_BUILT
  entry.

## Milestone gate

The M13 end gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

The gate holds. The successful installed-wheel evidence remains bound to hosted
run 31722258395 at `.claude/reviews/S35-sprint-review-pass-3.md:83`, covering
both packages across manylinux x86_64 and aarch64, musllinux x86_64, macOS
x86_64 and arm64, and Windows x86_64. S36 changed neither Python package nor
the wheel workflow. S36 separately satisfies its implementation-only gate, and
F-X006 owns the remaining fresh-version Rust registry release.

## Not found

- **Interaction**: the eight feature deltas compose without cross-story
  behavioral conflict. The publication remediation changes records only.
- **Duplication**: range, JSON, and path contracts have one shared owner. README
  has one snippet source, and one sample generator remains.
- **Layering**: no forbidden `oxml-*` dependency on `rdocx-*` or `rpptx-*` was
  introduced. CLI dependencies point inward.
- **Harness**: every plan and AS_BUILT entry declares unchanged output, matching
  the 28-entry integrated result.
- **Gate**: no technical S36 definition-of-done or M13 gate failure was found.
- **Docs**: plans, AS_BUILT, CURRENT, BACKLOG, SPRINT_PLAN, SPRINT_TRACKER, and
  HLD 03, 14, and 15 now agree on implementation completion and publication
  deferral.
- **Dependencies**: every new edge has a current consumer, and no unapproved
  dependency was added.
- **Surface**: every added public helper, facade operation, equality contract,
  command, and package surface is called for by a completed plan and tested.
- **Release safety**: the immutable 0.1.2 release is preserved, F-X006 requires
  a fresh version and separate approval, and S36 performed no publication.
- **Delivery records**: counts, estimates, actuals, statuses, plan checklists,
  feature reviews, AS_BUILT entries, and the one pending S37 story reconcile.
