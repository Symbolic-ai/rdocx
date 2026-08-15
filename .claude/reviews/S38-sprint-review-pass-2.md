# S38 sprint review, pass 2

**Reviewed**: `sprint/s38` at
`29ae0fe419f4e816d6990de87aed35b109872ac0` against merge base
`4adf3a6a728cb8bf9de0dfb782fdd2bfe5de4a57`, 75 files, 9,405 additions and
807 deletions. Crates: `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`, and
`rpptx-py`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Dispositions**: 0 fix-now, 0 tracked-follow-up, 1 human-action, 0 refuted

## Pass-1 disposition

### B1, resolved

`.claude/scratch/S38-run.json:25`
`.claude/commands/run-sprint.md:199`

The state authority now declares `phase=review`, as required before the sprint
review loop. The remediation changed no tracked source or delivery document,
so the complete sprint delta and exact reviewed HEAD remain unchanged. No
replacement process-state inconsistency was found.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Human action

### H1, obtain separate final approval for the stable release

`.claude/commands/release.md:75`
`.claude/plans/F-X008-design.md:119`

The clean sprint review completes the preparation gate, not the external
release gate. `/release v0.5.0` must report the exact reviewed SHA, package set,
remote, tag, and workflow, then obtain a separate explicit final approval
immediately before its first branch push, tag creation, tag push, or
publication action. Approval given earlier in the sprint does not satisfy this
boundary.

**Disposition**: human-action. No release mutation was performed by this pass.

## Milestone gate

The S38 preparation gate holds with evidence:

- GitHub reports PR 25 merged into `sprint/s38` at `6aade64`. Jon Stokes remains
  the author of all three contributor commits. The public maintainer comment
  thanks `@jonstokes`, explains the contribution's value, names the maintainer
  hardening, and preserves the separately approved release boundary. This
  satisfies the credit contract (`docs/sprints/CURRENT_SPRINT.md:41`).
- The complete feature state remains the clean F-X007 pass-18 state plus the
  clean F-X008 pass-1 preparation. The focused Word suites cover custom lists,
  rejected numbering mutation, hyperlinks, hard breaks, table geometry,
  unmodelled XML, namespace identity, schema ordering, bounded depth, and
  bounded high-count tab work (`docs/hld/14-development-backlog.md:1186`).
- Every stable publishable has a manifest-wired README with a purpose, example,
  or deprecation path. The fresh README runner compiled all twelve Rust
  examples and validated the exact seven-package inventory
  (`scripts/readme_doctests.py:27`, `scripts/readme_doctests.py:61`). The public
  0.5 construction migration remains explicit
  (`crates/rdocx-oxml/README.md:24`).
- Fresh Cargo metadata reports exactly eleven packages at 0.5.0, exactly seven
  publishable stable packages, and fifteen explicit incubating packages at
  0.1.3. The stable regression pins the nine internal pins, eleven lock
  entries, two Python versions, WASM literals, README requirements, exact
  publication set, and incubating isolation
  (`scripts/test_sprint_workflow.py:2871`). The publish workflow runs the stable
  and incubating preflights before its patched dry run
  (`.github/workflows/publish.yml:20`).
- The seven fresh local 0.5.0 archives each contain exactly one README. Their
  compressed sizes range from 3,021 to 99,885 bytes, below 10 MiB. Both WASM
  target checks passed, and no Python or WASM package gained crates.io
  publication eligibility (`crates/rdocx-wasm/Cargo.toml:13`,
  `crates/rdocx-py/Cargo.toml:5`, `crates/rpptx-py/Cargo.toml:5`).
- The full verification record remains passing at the exact current HEAD with
  the harness unchanged (`.claude/scratch/S38-run.json:37`). Fresh pass-2 probes
  also passed all 37 workflow tests, all twelve README examples, both WASM
  target checks, all 28 hash entries, formatting, prose, generated-skill sync,
  and diff checks.
- The authoritative HLD set describes current package preservation, public 0.5
  migration, tests, exact release families, and separate approval mechanism
  (`docs/hld/04-opc-and-packaging.md:237`,
  `docs/hld/10-bindings-spec.md:190`,
  `docs/hld/12-testing-strategy.md:428`,
  `docs/hld/14-development-backlog.md:1218`,
  `docs/hld/15-build-and-toolchain.md:208`). No implementation contradiction
  falls outside either plan's HLD impact list.
- The durable records and state authority agree. F-X007 is completed with its
  owner cleared, completed plan, tracker row, and AS_BUILT entry. F-X008 remains
  approved and owned, reviewed in run state, and in progress in both delivery
  trackers without premature completion ledgers
  (`docs/sprints/CURRENT_SPRINT.md:28`, `docs/sprints/BACKLOG.md:294`,
  `docs/sprints/SPRINT_TRACKER.md:220`, `docs/sprints/AS_BUILT.md:5529`,
  `.claude/scratch/S38-run.json:3`, `.claude/scratch/S38-run.json:12`).

The external release gate is intentionally pending. Local and remote
`v0.5.0` tags, the GitHub release, crates.io 0.5.0, npm 0.5.0, and both PyPI
0.5.0 projects remain absent. This matches the prepared-state contract
(`docs/hld/15-build-and-toolchain.md:219`).

## Not found

No cross-feature interaction defect, duplication, dependency layering
violation, undeclared harness delta, weak milestone gate, HLD drift,
unauthorized dependency, unrequested public surface, migration gap, README
inventory gap, archive violation, contributor-credit loss, release mutation,
registry publication, tracked follow-up, or refuted finding was found.
