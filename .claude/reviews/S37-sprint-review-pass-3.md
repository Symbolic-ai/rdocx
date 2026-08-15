# S37 sprint review, pass 3

**Reviewed**: `sprint/s37` at
`6b69df4d65924027c76d6de230a7de9b8d74a502` against merge base
`6cb41a282f52aae2396bd619ec3b2a25e0f7a1a1`, 41 files, 592 insertions and
148 deletions. Crates: `oxml-cli-support`, `oxml-core`, `oxml-drawing`,
`oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-render`, and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Run-sprint disposition

- `fix-now`: none.
- `tracked-follow-up`: none. npm and PyPI registry publication remain outside
  S37.
- `human-action`: none for S37.
- `refuted`: none.

## Earlier finding

### B1, resolved

The durable delivery record now proves the required approval boundary. The
sprint definition of done requires separate final approval at the reviewed SHA
before tag creation or push at `docs/sprints/CURRENT_SPRINT.md:39`. The
completed plan records receipt of that approval at
`.claude/plans/F-X006-design.md:152`. AS_BUILT binds the annotated tag to
reviewed SHA `805680ab8a6dadd4d4247471a81cbb21b88a3196` at
`docs/sprints/AS_BUILT.md:5500`, then states that the user gave separate final
approval at that SHA immediately before the branch push, tag creation, tag
push, and publication workflow at `docs/sprints/AS_BUILT.md:5503`.

This directly resolves pass 2 B1. The remediation changes only the approved
plan checklist and AS_BUILT evidence. It adds no source, manifest, workflow,
tag, registry, or publication mutation.

## Sprint definition of done

All five S37 definition-of-done items hold.

- Cargo metadata reports exactly 11 `workspace` preparation members at 0.4.1,
  15 `incubating` preparation members at 0.1.3, 14 publishable incubating
  packages, and the exact 21-package publishable union. The package boundary is
  documented at `docs/hld/15-build-and-toolchain.md:217`.
- Full verification passed at the reviewed release SHA, all 28 hashes remained
  unchanged, and GitHub Actions run `31762653847` completed successfully for
  tag `rpptx-v0.1.3` at `docs/sprints/AS_BUILT.md:5517`. Fresh pass-3 checks
  also passed all 36 sprint-workflow tests, the 28-entry hash harness, generated
  adapter validation, prose validation, and diff hygiene.
- The separate final approval is now recorded at the exact external mutation
  boundary as described in the resolved finding above.
- Fresh crates.io checks report all 14 exact 0.1.3 versions present and
  unyanked. Every exact owner set contains only `mantissaman (Atul Sharma)`,
  matching the named ownership contract at
  `docs/hld/15-build-and-toolchain.md:142`. Run `31762653847` is completed with
  conclusion `success` at release SHA
  `805680ab8a6dadd4d4247471a81cbb21b88a3196`. Its publication and GitHub
  release jobs succeeded, the stable allowlist was skipped, and the incubating
  allowlist succeeded.
- `rpptx-v0.1.3` is an annotated local and remote tag whose object peels to the
  reviewed release SHA. The immutable local and remote `rpptx-v0.1.2` tag still
  peels to `27a8bb8aa494759568d40bf66c167c214e759500`, its original 12 registry
  versions remain present and unyanked, and the two later packages remain absent
  at 0.1.2. Both scoped npm packages remain absent. The immutable history and
  no-overwrite result are recorded at `docs/hld/03-architecture.md:136`, and
  the npm exclusion is recorded at `docs/sprints/AS_BUILT.md:5502`.

The delivery ledgers reconcile. F-X006 is done and unowned at
`docs/sprints/CURRENT_SPRINT.md:24`, has one completed tracker row at
`docs/sprints/SPRINT_TRACKER.md:218`, and has one AS_BUILT entry at
`docs/sprints/AS_BUILT.md:5488`. BACKLOG reports all 160 stories done and no
pending or carried work at `docs/sprints/BACKLOG.md:33`. The completed feature
plan lists exactly HLD 03, HLD 14, and HLD 15 at
`.claude/plans/F-X006-design.md:98`, and those three files consistently describe
the published 0.1.3 family, unpublished `rpptx-wasm`, immutable 0.1.2 history,
and future fresh-version release authority.

## Milestone gate

The F-X006 backlog gate is that all 14 incubating packages resolve from
crates.io at 0.1.3 with the expected owner and the GitHub release targets the
reviewed sprint SHA at `docs/hld/14-development-backlog.md:1183`. The fresh
registry, owner, workflow, release, and annotated-tag checks above show that the
gate holds.

The M13 end gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`. The gate remains
satisfied by successful hosted run `31722258395`, whose reviewed evidence covers
both packages across all six target families at
`.claude/reviews/S35-sprint-review-pass-3.md:83`. A fresh pass-3 GitHub query
confirmed that run is still completed with conclusion `success`. S37 changes no
Python package or wheel workflow.

## Not found

- **Interaction**: the 0.1.3 preparation, tagged publication, final HLD state,
  release evidence, and completion ledgers compose without a package, version,
  status, or approval-boundary conflict.
- **Duplication**: no duplicate helper or package contract was introduced.
  `oxml-layout` remains the sole bundled-font asset owner and `rdocx-layout`
  contains no copied inventory.
- **Layering**: no dependency edge changed. Fresh dependency trees confirm that
  `oxml-cli-support` has no document-family dependency and `rpptx-cli` depends
  inward on the shared helper and presentation family. No forbidden cross-family
  edge was introduced.
- **Harness**: no rendering baseline changed. The plan, AS_BUILT record,
  workflow evidence, and fresh independent check all report 28 unchanged
  entries.
- **Gate**: no S37 definition-of-done, F-X006 backlog gate, or M13 end-gate
  failure was found.
- **Docs**: CURRENT_SPRINT, BACKLOG, SPRINT_PLAN, SPRINT_TRACKER, AS_BUILT, the
  completed plan, and HLD 03, 14, and 15 agree on the 14 published packages,
  15-member preparation group, 0.1.3 version, immutable 0.1.2 release, separate
  approval, and no npm publication.
- **Dependencies**: no dependency was added or removed, and all internal
  incubating pins are coherent at 0.1.3.
- **Surface**: no public Rust API, rendering behavior, document behavior, npm
  registry surface, or PyPI registry surface changed.
- **Release safety**: the fresh and immutable tag objects agree locally and
  remotely, the publication workflow selected only the incubating allowlist,
  and the stable family remains at 0.4.1.
- **Delivery records**: story count, estimate, actual, status, ownership, plan
  checklist, feature review, AS_BUILT entry, and HLD impact list reconcile.
