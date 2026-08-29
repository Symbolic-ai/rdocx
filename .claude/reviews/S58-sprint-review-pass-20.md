# S58 sprint review, pass 20

**Reviewed**: `sprint/s58` at
`401dcb869e784d0b733a454b8b58e3e0a9eee133` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 205 files, 24,154 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`.
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This twentieth pass is the explicitly authorized post-publication review after
the F-X068 release gate and completion ledgers. It audits a new external
release and tracked completion delta rather than repeating pass 19 over an
unchanged state. Recording the reason here satisfies the later-pass exception
required by `.claude/commands/sprint-review.md:45` and
`.claude/commands/sprint-review.md:86`.

## Blocking

### B1, the HLD still describes shared 0.8.0 as unpublished

`docs/hld/03-architecture.md:545`

All five HLD files in F-X068's approved impact list still describe the shared
0.8.0 family as prepared or awaiting publication and 0.7.0 as the latest
complete published family. The same stale state appears in the binding contract
at `docs/hld/10-bindings-spec.md:727`. The testing strategy's release-evidence
sequence records 0.7.0 at `docs/hld/12-testing-strategy.md:1192`, then jumps to
the failed stable attempt at `docs/hld/12-testing-strategy.md:1198` without the
completed 0.8.0 gate. The F-X068 story says publication remains gated at
`docs/hld/14-development-backlog.md:3314`, and both publication sections still
name 0.7.0 as latest at `docs/hld/15-build-and-toolchain.md:281` and
`docs/hld/15-build-and-toolchain.md:402`.

That contradicts the completed release record, which says all 15 packages were
published from reviewed SHA
`7f4414b0aeef1ec2cbae75fcb5aa96ab6dee6d70` at
`docs/sprints/AS_BUILT.md:9994` through `docs/sprints/AS_BUILT.md:10011`, and it
misstates the registry dependency that F-X069 is now approved to consume. The
fix must update exactly the five plan-listed HLD files to current
post-publication reality. It must name 0.8.0 as the latest complete incubating
publication from the immutable annotated `rpptx-v0.8.0` tag at the reviewed
SHA, retain the stable 0.11.0 partial-attempt evidence and 0.10.1 complete-family
boundary, keep `rpptx-wasm` unpublished, and record the verified release gate
without turning the HLD into change-history prose.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this post-publication dependency-prefix checkpoint.
F-X069, F-X070, and F-X031 remain pending at
`docs/sprints/CURRENT_SPRINT.md:48` through
`docs/sprints/CURRENT_SPRINT.md:50`. The sprint definition still requires the
complete stable 0.11.1 recovery, separately approved yanks, and final
branch-protection work at `docs/sprints/CURRENT_SPRINT.md:83` through
`docs/sprints/CURRENT_SPRINT.md:91`.

The F-X068 release gate itself holds. The annotated local and remote
`rpptx-v0.8.0` tag dereferences to reviewed SHA
`7f4414b0aeef1ec2cbae75fcb5aa96ab6dee6d70`. GitHub Actions run 33258210706
completed the output-stability, metadata, notes, archive, exact 15-package
incubating publication, and GitHub Release jobs while skipping stable
publication, matching `docs/sprints/AS_BUILT.md:10001` through
`docs/sprints/AS_BUILT.md:10011`. All 15 registry entries resolve at 0.8.0, are
unyanked, and have sole owner `mantissaman (Atul Sharma)`.
`rpptx-wasm@0.8.0` is absent, and the 2,016-byte GitHub release body is
byte-identical to the committed changelog render. This dependency publication
does not establish the final M20 or sprint end gate.

## Not found

- **Publication evidence, 0 findings**: the release run, exact annotated tag
  target, selected 15-package registry set, owner inventory, release body,
  unpublished WASM exclusion, and stable registry graph independently agree
  with the release record at `docs/sprints/AS_BUILT.md:10001` through
  `docs/sprints/AS_BUILT.md:10011`.
- **Completion ledgers, 0 findings**: F-X068 is done with no owner in the active
  sprint at `docs/sprints/CURRENT_SPRINT.md:47`, done in the backlog at
  `docs/sprints/BACKLOG.md:523`, and recorded once with matching size and
  actuals at `docs/sprints/SPRINT_TRACKER.md:345`. Its design plan is completed
  with every release checklist item ticked at
  `.claude/plans/F-X068-design.md:112` through
  `.claude/plans/F-X068-design.md:121`.
- **Contribution inventory and notifications, 0 findings**: the selected
  shared-family inventory is empty, so no comment was due and no issue or pull
  request state changed at `docs/sprints/AS_BUILT.md:10013` through
  `docs/sprints/AS_BUILT.md:10018`. Issues 53 and 54 and PRs 55 through 58
  remain assigned to the later stable recovery.
- **F-X069 dependency readiness, 0 additional findings**: F-X068 is completed
  in sprint state at `.claude/scratch/S58-run.json:192` through
  `.claude/scratch/S58-run.json:203`, and F-X069 names it as a dependency at
  `.claude/plans/F-X069-design.md:6`. The published registry boundary is ready,
  but the dependency prefix is not clean for the next wave until B1 restores
  authoritative HLD consistency.
- **Interaction, 0 additional findings**: the released shared direction carrier
  matches the stable recovery dependency and does not change the reviewed S58
  runtime behavior. The only post-publication interaction defect is the stale
  HLD state in B1.
- **Duplication, 0 findings**: finalization adds one AS_BUILT entry and one
  tracker row. It adds no second release path, registry proof, contribution
  inventory, or completion record.
- **Layering, dependencies, and surface, 0 findings**: the release finalization
  changes only the design status and delivery ledgers. It adds no runtime
  dependency edge, crate, module, feature flag, public API, binding method, or
  publication authority.
- **Harness and gate, 0 additional findings**: full verification at the release
  SHA records all 49 hashes unchanged at `.claude/scratch/S58-run.json:581`
  through `.claude/scratch/S58-run.json:585`, and the completion record agrees
  at `docs/sprints/AS_BUILT.md:10025` through
  `docs/sprints/AS_BUILT.md:10032`. M20 remains open rather than being inferred
  from the successful dependency publication.
- **Docs, 0 additional findings**: the AS_BUILT entry accurately distinguishes
  the published 15-package family, unpublished `rpptx-wasm`, empty contribution
  inventory, and later stable consumer at `docs/sprints/AS_BUILT.md:9988`
  through `docs/sprints/AS_BUILT.md:10036`. The authoritative HLD mismatch is
  fully captured by B1.
- **Package, legal, font, and assets, 0 findings**: the successful release
  workflow repeated archive verification before the real incubating allowlist,
  and the completion record binds that check to the release at
  `docs/sprints/AS_BUILT.md:10025` through
  `docs/sprints/AS_BUILT.md:10030`. F-X068 changed carriers and release evidence,
  not the reviewed package inventory.
