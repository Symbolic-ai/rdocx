# S37 sprint review, pass 2

**Reviewed**: `sprint/s37` at
`e96c48b9d349a0f94ded1519414b95805c91bafb` against merge base
`6cb41a282f52aae2396bd619ec3b2a25e0f7a1a1`, 40 files and 645 changed lines.
Crates: `oxml-cli-support`, `oxml-core`, `oxml-drawing`, `oxml-layout`,
`oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`, and
`rpptx-wasm`.
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, The completion record does not evidence receipt of final approval

`docs/sprints/CURRENT_SPRINT.md:39`
`docs/sprints/AS_BUILT.md:5499`
`.claude/plans/F-X006-design.md:152`

The sprint definition of done requires the release command to receive separate
final approval at the reviewed SHA before it creates or pushes the tag. The
completed plan records only that `/release` requested that approval, and the
AS_BUILT entry records the version choice, tag, workflow, registry results, and
npm exclusion without stating that the distinct approval was received before
the first external mutation. Publication success does not prove the approval
boundary by itself. Record the received separate final approval at
`805680ab8a6dadd4d4247471a81cbb21b88a3196`, if that event occurred, so the
completed delivery record proves this DoD item rather than inferring it.

## Should-fix

None.

## Nice-to-have

None.

## Prior pass

Pass 1's pending external gate is resolved. The reviewed release was performed
after the clean preparation review, and the external package, workflow, tag,
release, owner, and no-npm observations now succeed. B1 concerns only the
missing persistent evidence for the separate approval boundary.

## Milestone gate

The F-X006 backlog gate is: "all 14 incubating packages resolve from crates.io
at 0.1.3 with the expected owner, and the GitHub release targets the reviewed
sprint SHA."

The backlog gate holds:

- The crates.io API reports all 14 exact 0.1.3 versions present and unyanked.
  Each exact owner set contains only `mantissaman (Atul Sharma)`.
- GitHub Actions run `31762653847` is completed with conclusion `success` for
  tag `rpptx-v0.1.3` at
  `805680ab8a6dadd4d4247471a81cbb21b88a3196`. Its `Publish to crates.io` and
  `GitHub Release` jobs both succeeded. The stable allowlist was skipped and
  the incubating allowlist succeeded.
- `rpptx-v0.1.3` is an annotated local and remote tag whose object peels to
  `805680ab8a6dadd4d4247471a81cbb21b88a3196`. The non-draft,
  non-prerelease GitHub release exists for that tag.
- The immutable `rpptx-v0.1.2` local and remote tag still peels to
  `27a8bb8aa494759568d40bf66c167c214e759500`. Its original 12 registry
  versions remain present and unyanked, while `oxml-cli-support` and
  `rpptx-cli` remain absent at 0.1.2.
- Both `@tensorbee/rdocx-wasm` and `@tensorbee/rpptx-wasm` remain absent from
  the npm registry. No npm publication authority was added.

The other sprint records align. F-X006 is completed in the plan, sprint state,
CURRENT_SPRINT, BACKLOG, AS_BUILT, and SPRINT_TRACKER. BACKLOG reports 160 of
160 stories done. Exactly HLD 03, HLD 14, and HLD 15 describe the published
14-package 0.1.3 family, unpublished `rpptx-wasm`, immutable 0.1.2 history, and
future fresh-version release authority. Five focused release regressions pass,
all 28 hashes match, generated adapters are in sync, and prose and diff hygiene
pass.

## Not found

- Interaction: version preparation, publication, final HLD state, and delivery
  ledgers compose without a package or status conflict apart from B1.
- Duplication: no duplicate helper or asset inventory. `oxml-layout` remains
  the sole bundled-font owner and `rdocx-layout` contains no copied inventory.
- Layering: no dependency edge changed and no forbidden cross-family edge was
  introduced.
- Harness: no baseline changed. The design, AS_BUILT entry, recorded full gate,
  workflow run, and independent check all report 28 unchanged entries.
- Docs: the three approved HLD files consistently describe current published
  state, the two immutable releases, the 15 prepared versus 14 published
  boundary, and corrected archive ownership. The separate approval evidence
  gap is isolated in B1.
- Dependencies: no dependency was added or removed.
- Surface: no public Rust API or rendering behaviour changed.
