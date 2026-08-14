# S38 sprint review, pass 1

**Reviewed**: `sprint/s38` at
`29ae0fe419f4e816d6990de87aed35b109872ac0` against merge base
`4adf3a6a728cb8bf9de0dfb782fdd2bfe5de4a57`, 75 files, 9,405 additions and
807 deletions. Crates: `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`, and
`rpptx-py`.
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have
**Dispositions**: 1 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Blocking

### B1, sprint state did not advance to the review phase

`.claude/scratch/S38-run.json:25`
`.claude/commands/run-sprint.md:199`

The exact-HEAD full verification is recorded, F-X007 is completed, and F-X008
is correctly retained as reviewed. The state authority nevertheless still
declares the sprint phase as `verification` while this sprint review is being
run. The canonical sequence requires `set-phase S38 review` after finalising
the delivery record and before the review loop. Set the state phase to
`review`, then run a fresh independent pass at the same tracked HEAD.

**Disposition**: fix-now.

## Should-fix

None.

## Nice-to-have

None.

## Human action

### H1, the stable release remains behind its separate final approval

`.claude/commands/release.md:75`
`.claude/plans/F-X008-design.md:119`

No release mutation belongs in this review. After a clean sprint-review pass is
recorded at the fully verified SHA, `/release v0.5.0` must report the exact SHA,
package set, remote, tag, and workflow, then obtain a separate explicit final
approval immediately before the first branch push, tag creation, tag push, or
publication action. Earlier sprint approvals do not satisfy this boundary.

**Disposition**: human-action after B1 is remediated and a clean pass is
recorded.

## Milestone gate

The S38 gate requires PR 25 and its contributor credit, package-preserving Word
authoring APIs, useful documentation for every stable crate, twelve compiling
Rust examples, a full unchanged-harness verification, seven bounded stable
archives, and a separately approved 0.5.0 release
(`docs/sprints/CURRENT_SPRINT.md:39`).

The preparation portion holds:

- GitHub reports PR 25 merged into `sprint/s38` at `6aade64`. Jon Stokes remains
  the author of all three contributor commits. The public maintainer comment
  thanks `@jonstokes`, explains why the contribution is valuable, names the
  hardening, and preserves the later release boundary. The sprint contract
  requires that exact credit and note (`docs/sprints/CURRENT_SPRINT.md:41`).
- The focused `rdocx-oxml` suite passed 164 tests and its README doctest. The
  focused `rdocx` suite passed 69 unit, 81 integration, 17 regression, and two
  doctests. These include custom lists, rejected numbering mutation,
  hyperlinks, hard breaks, table geometry, namespace preservation, schema
  ordering, bounded depth, and the 10,000-tab work gate described by the HLD
  (`docs/hld/14-development-backlog.md:1207`).
- Every stable publishable has a manifest-wired README and a clear purpose,
  example, or deprecation path. The README runner compiled all twelve Rust
  examples and validates the seven-package inventory
  (`scripts/readme_doctests.py:27`, `scripts/readme_doctests.py:61`). The
  `rdocx-oxml` README explains the intentional 0.5 construction migration
  (`crates/rdocx-oxml/README.md:24`).
- Cargo metadata reports exactly eleven inherited packages at 0.5.0, exactly
  seven publishable stable packages, and fifteen explicit incubating packages
  at 0.1.3. The stable regression pins the eleven lock entries, nine workspace
  pins, two Python versions, WASM literals, README requirements, seven-package
  publication set, and incubating isolation
  (`scripts/test_sprint_workflow.py:2871`). The workflow runs that regression
  with the incubating regression before the patched dry run
  (`.github/workflows/publish.yml:20`).
- Fresh local 0.5.0 archives for all seven stable packages each contain exactly
  one README and range from 3,021 to 99,885 bytes, below the 10 MiB limit. The
  recorded full verification passed at the exact reviewed HEAD with the harness
  unchanged (`.claude/scratch/S38-run.json:29`). An independent focused rerun
  also passed all 37 workflow tests, both WASM target checks, the README runner,
  and all 28 deterministic hashes.
- The HLD set reflects current intent and mechanism for package preservation,
  the public 0.5 migration, the test gate, the exact release-family metadata,
  and the separate approval boundary (`docs/hld/04-opc-and-packaging.md:237`,
  `docs/hld/10-bindings-spec.md:190`,
  `docs/hld/12-testing-strategy.md:428`,
  `docs/hld/14-development-backlog.md:1218`,
  `docs/hld/15-build-and-toolchain.md:208`). No unlisted HLD contradiction was
  found.
- The durable records consistently mark F-X007 done and completed with its
  owner cleared, design completed, tracker row, and AS_BUILT entry. They retain
  F-X008 as approved, owned, reviewed in run state, and in progress without a
  premature tracker or AS_BUILT entry
  (`docs/sprints/CURRENT_SPRINT.md:28`, `docs/sprints/BACKLOG.md:294`,
  `docs/sprints/SPRINT_TRACKER.md:220`, `docs/sprints/AS_BUILT.md:5529`,
  `.claude/scratch/S38-run.json:3`, `.claude/scratch/S38-run.json:12`).

The external portion is deliberately pending. Local and remote `v0.5.0` tags,
the GitHub release, all seven crates.io 0.5.0 versions, the npm 0.5.0 package,
and both PyPI 0.5.0 projects are absent. `origin/sprint/s38` remains behind the
reviewed preparation commit. This is the correct pre-approval state, not a met
external release gate (`docs/hld/15-build-and-toolchain.md:219`).

## Not found

No cross-feature interaction defect, helper duplication, dependency layering
violation, undeclared harness delta, gate weakness, HLD drift, unauthorized
dependency, unrequested public surface, broken migration guidance, incomplete
README inventory, package-size violation, contributor-credit loss, release-tag
mutation, crates.io publication, npm publication, or Python publication was
found. No tracked follow-up or refuted finding is needed.
