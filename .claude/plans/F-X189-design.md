# F-X189, Prepare Word 0.16.0 and PowerPoint 0.14.0 families

**Status**: completed
**Sprint**: S90
**Size**: L
**Depends on**: F-278, F-279, F-280, F-281, F-282, F-X179, F-X180, F-X181, F-X182, F-X183, F-X184, F-X185, F-X186, F-X187, F-X188, F-X190, F-X191

## Problem

The user requests Rust and Python publication after S90. Cargo.toml:35 and
both Python project manifests still carry Word0.15.0 and PowerPoint0.13.1.
CHANGELOG.md:3 has no Unreleased changes. These versions already have remote
annotated tags and GitHub releases at the S88 main merge
9d019472f7e6b95ac4ba0770c4dee35dcbc28f0e. Read-only registry queries also
confirm both main crate and Python versions are published. Existing S88
preparation and build evidence cannot authorize publication of S90 source.

The user explicitly approved new Word0.16.0 and shared/PowerPoint0.14.0
preparation and this story's design, review, progress and handoff records.
This approval does not replace either family's final publication approval.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X189, Prepare Word 0.16.0 and PowerPoint 0.14.0 families (L)", both family preparation and dependency barriers.
- `docs/hld/15-build-and-toolchain.md`, "Packaging", "Publishing", "Release process" and "CI job matrix", exact package sets, source provenance, build-only rehearsal and separate publication.
- `docs/hld/10-bindings-spec.md`, "Packaging" and "CI", Python metadata, installed runtime, supported floor and strict typing.
- `.claude/commands/release-notes.md`, "Evidence" and "Validate and review", family-specific notes and authenticated contribution inventory.
- `.claude/commands/release.md`, "Family contract", "Preconditions" and "Final approval", immutable tags and exact closed-main release authority.

## Intake and sequencing

Design uses the run-sprint batch route. No unanswered scope question remains,
but unfinished prerequisites still block implementation. Complete every
listed dependency through its approved lifecycle before claiming exclusive
wave18. The user moved remaining bibliography catalogue work to F-X192 and
carried F-283 to S91. F-282 must complete its revised measured-subset contract
before preparation, without a full catalogue claim.
Reviewed X180, X181 and X183 require dependency-prefix delivery reconciliation
before this consumer starts. Source, Cargo, manifests, README measurements and
HLD edits remain exclusive. The local preparation is completed before final
integrated verification and sprint review. The hosted build-only rehearsal
runs at the resulting reviewed and pushed sprint SHA before close.

The read-only contribution map is
`/private/tmp/S90-release-contribution-readiness-map.md`, SHA256
e9b53254f091635527ea293988be797b961bdb68a132d2c4b98ee184d642fbab.
It records pending acceptance explicitly and is not a completion authority.
Fresh intake includes PR290 and Issue292 within F-X188. Reconcile their
verified contribution evidence in the release inventory. The user corrected
the exclusion to Issue281, which remains open for F-184. Issue291 is included
and its separately verified field snapshot correction must complete before
release preparation. Issue264 remains excluded. Updated stacked PRs require
selective acceptance rather than automatic adoption of the complete stack.
The user froze S90 contribution intake at PR293. Do not add later GitHub
issues or pull requests to this release preparation scope.
PR293 is included through F-X191 in exclusive wave17, with independent
verification required before completion.
Complete its namespace-only reader correction before preparation and credit
Pedro Assumpcao (`pedroassumpcao`) in the authenticated inventory. Preserve
current archive policy without contributor platform overrides.
Refresh the inventory and exact previous family tags before writing claims.
Issue264/F-X178 remain excluded, including release notifications. Preserve
historical changelog credit without treating it as new S90 work. Issue281
remains open for the F-184 roadmap decision.

## Approach

Prepare the stable workspace group and exact seven published Word packages
at0.16.0. Prepare the exact fifteen shared OOXML/PowerPoint published packages
and rpptx-py at0.14.0. Update corresponding internal workspace pins,
Cargo.lock, both pyproject.toml versions, current-version assertions and
README installation examples. Keep publish=false bindings and WASM packages
outside crates.io allowlists. Preserve rpptx-wasm's separate existing npm
boundary. Inspect inherited oxml-py-support and rdocx-wasm carriers explicitly
rather than introducing another version group. Never change external tool
versions such as wasm-pack0.15.0 by broad string replacement. No new runtime
API, production file, module, crate, dependency, feature flag or generic.

Use existing unified-release-family and release-notes checks for
`v0.16.0` and `rpptx-v0.14.0`. Extend the existing workflow test module with
the named release regression, retaining immutable historical release tests.
Current-family policy assertions advance only where they describe current
carriers. Re-measure every affected published archive and README inventory
from source, authenticate normalized payloads and enforce the10MiB ceiling.
Run the actual22-package locally patched publication dry run without upload.
The Word packages consume the newly prepared shared family, so final release
ordering is rpptx-v0.14.0 before v0.16.0 where verified dependency pins require
it. Each tag still has its own final approval.

Write two meaningful family sections through release-notes. Derive claims
from completed reviewed acceptance across the full range since each previous
family tag, including S89. Credit authenticated hadim reports/contributions
and separately changjoon-park's accepted PR269 where that shared-family change
belongs. Every included issue and PR has a direct link, specific outcome and
direct or hardened-equivalent classification. Do not claim unmerged PRs were
merged unchanged, count stacked work twice, attribute independently authored
features to reporters, or claim spreadsheet implementation. Describe actual
compatibility changes, including required public layout fields and reviewed
rendering deltas, without asserting universal Word pixel parity. Final
bibliography claims state the reviewed F-282 admission boundaries. Remaining
full catalogue parity belongs to F-X192 and navigation to carried F-283.

Build current local Python wheels and source distributions, inspect exact
metadata and install them into clean Python3.9 and3.12 environments. Run each
family's priority runtime suites and exact mypy2.3.0/stubtest under3.12.
Retain documented oracle-dependent exclusions only where the approved hosted
environment lacks those tools, with their pinned native/CI gates retained.
Check both WASM bindings and the separate rpptx-wasm carrier contract.

The actual default-size rider exposed an inherited test-only omission. The
rpptx-wasm manifest and HLD already require wasm-opt125 with both bulk-memory
and nontrapping-float-to-int enabled. Reconcile the existing exact size test's
expected arguments, actual invocation and normalized path indices with that
contract. Extend its existing rejection case to refuse omission of the
nontrapping flag. Preserve the real compiled failures, then run the focused
negative control, actual ignored size/roundtrip gate and bundler/npm riders.
No runtime algorithm, manifest, tool version, wrapper or production API changes.
A separate incremental ALL pass reviews this bounded test reconciliation.
The next real pipeline execution exposed an abbreviated test-only version
literal. Match the exact official Binaryen identity already required by CI and
HLD, `wasm-opt version 125 (version_125)`, in the validator and fixture. Add an
abbreviated-identity refusal case and reset it before existing gzip checks.
Preserve that actual failure and obtain a distinct incremental ALL pass.

After the final integrated full gate, clean sprint review and sprint push,
dispatch wheels.yml at that exact SHA in its existing build-only mode. It
builds both Python projects and cannot reach registry or GitHub release jobs.
Inspect six cp39-abi3 wheels and one source distribution per family, current
metadata, downloaded provenance and clean installed checks. Current manual
dispatch does not run tag-only CLI asset jobs. Do not claim it produced CLI
archives. Validate each family's six-CLI/six-wheel/one-sdist and selected
fourteen-asset release contracts from workflow source and applicable tests.
Actual per-family six CLI archives, six wheels, one sdist and SHA256SUMS are
verified after the authorized tag workflow under release.

Update current release mechanism/version prose in exactly the listed HLD
files and current repository guidance, preserving immutable historical facts.
Do not report future publication as completed preparation. After close,
release independently checks exact main/tree provenance, full verification,
all selected registry versions, remote tag absence, owner roles and trusted
publisher identity. Only then request separate final approval and publish.

## Rejected alternatives

- Reusing historical tags would alter immutable release history.
- Publishing source that still names old versions would fail registry uniqueness and family metadata checks.
- Using S88 build evidence for S90 would leave current binaries unverified.
- A Python-only tag or mixed family release would abandon the unified family contract.
- Publishing during run-sprint would bypass closed-main review and final approval.

## Test plan

**Test gate**: release regression.
`s90_release_families_match_reviewed_versions` in
scripts/test_sprint_workflow.py proves the exact new family contract fails
against claimed Base carriers, then passes both selected allowlists, versions
and rendered notes. Missing new production APIs are not fail-before evidence.

| Category | Test | Asserts |
|---|---|---|
| release regression | `s90_release_families_match_reviewed_versions` | Both exact crate sets, binding/project carriers and reviewed rendered notes agree with approved tags |
| package | Actual locally patched workspace publication dry run | All22 archives verify against local source, exact selected inventories, measured payloads and below10MiB bounds |
| Python | Clean installed current wheels on3.9 and3.12 | Runtime priority suites, exact metadata and Python3.12 mypy2.3.0/stubtest for both distributions |
| integration | Version, README, workflow and WASM checks | Current literals update without changing external tools, historical contracts or npm publication authority |
| post-integration artifact | Final reviewed-SHA hosted build-only rehearsal | Twelve wheels and two sdists across both projects have exact metadata and provenance, no publication |
| post-integration release | Final full sprint and read-only main gates | Hash baseline and declared sprint delta remain unchanged by preparation, clean review and close provenance retained |

The local scoped gate and independent zero-finding microscope precede prepare
and integration. The final hosted rehearsal is a pre-close gate, not a worker
push or a fictitious local completion result. Failed artifact, metadata or
registry checks remain failures. No release tag is created in this story.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting/version strings: read release.md and HLD15. Inspect each manifest, pin, lockfile, README and policy diff. Run both family preflights, complete locally patched package dry run, final full gate and separate immediate exact-SHA approvals before tags.
- Public published API compatibility: read HLD10 and structural rules. Notes document actual preceding pre-1.0 API changes. Authenticate every affected archive and the10MiB limit without adding runtime surface.
- Crate dependency graph: read HLD03. Only existing internal version edges change. Prove shared-family publication order and preserve forbidden dependency direction and exact allowlists.
- WASM/PyO3: read HLD10. Rebuild both actual bindings, clean Python3.9/3.12 installs, pinned typing/stubs and both WASM checks. Exclude Python binding crates from linked workspace Rust tests.
- New records: explicitly approved by the user. Existing source/test entrypoints only, no new production module/file/dependency/trait/generic. No native Word capture is required for release metadata.

## Hash harness

Expected unchanged relative to this story's claimed integrated Base, all49
entries. Retain earlier separately reviewed sprint output deltas and goldens.
Version preparation must not change rendering or replace the baseline.

## Implementation checklist

- [x] Confirm old tags/releases and obtain approval for both new versions and workflow records.
- [x] Complete every formal prerequisite and claim exclusive wave18.
- [x] Capture genuine new-version contract failure against the immutable Base.
- [x] Update exact carriers, pins, notes, policy, inventories and listed HLD files.
- [x] Pass scoped package/binding/typing checks and independent microscope for local preparation.

## Post-integration gates

The implementation checklist records local release preparation. Completing this
story does not claim sprint closure, a hosted rehearsal or publication.
After integration, run final full sprint verification and clean sprint review,
then the SHA-bound hosted build-only rehearsal before close. These mandatory
post-integration gates retain the exact artifact obligations in the test plan.
They cannot execute against a worker SHA as a substitute for the final reviewed
and pushed sprint SHA.

After the sprint closes, run each family's read-only release preflight and
obtain its separate final publication approval at reviewed main. The release
command requires this preparation story to be done and its plan completed
before creating either tag. Publication evidence belongs to the release report,
not a prospective checked implementation item or a post-release main edit.

## Open questions

None for scope or workflow-file permission. The user selected both families
and proposed minor versions. Every selected registry version and current
artifact must still be independently checked. A conflicting registry version
or genuine contract gap is reported before changing approved versions or
weakening acceptance. Publication approval remains a separate final decision.
